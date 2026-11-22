use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store {
    pub conn: Mutex<Connection>,
}

#[derive(Debug, Serialize)]
pub struct RepoView {
    pub id: i64,
    pub path: String,
    pub mode: String,
    pub head_sha: String,
    pub branch: String,
    pub state: String,
}

pub type EventRecord = (i64, i64, i64, String, String);
pub type JobRecord = (i64, String, i64, String, i64);
pub type FindingRecord = (String, String, String, i64, String);
/// (id, aliases, ecosystem, ranges, severity, fixed, modified, withdrawn)
pub type VulnRecord = (String, String, String, String, String, String, i64, i64);
/// (id, src_repo, dst_repo, kind, refspec, path_glob, cron)
pub type PolicyRecord = (
    i64,
    Option<i64>,
    Option<i64>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub fn unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Relógio LRU: `unix()` em segundos com fração monotônica por processo.
/// `unix()` sozinho tem granularidade de 1 s — um `touch` no mesmo segundo
/// da inserção empata e o desempate por `last_used` escolhia a vítima errada
/// (worktree renovado era despejado em vez do menos usado). O contador
/// (`AtomicU64` dentro do mesmo segundo) garante ordem total.
fn lru_clock() -> i64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed) % 1_000_000;
    unix() * 1_000_000 + seq as i64
}

impl Store {
    pub fn register_scan_root(&self, path: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO scan_roots(path, last_scan) VALUES(?1, ?2)
             ON CONFLICT(path) DO UPDATE SET last_scan=excluded.last_scan",
            rusqlite::params![path, unix()],
        )?;
        Ok(())
    }

    pub fn list_scan_roots(&self) -> anyhow::Result<Vec<String>> {
        let c = self.conn.lock().unwrap();
        let mut statement = c.prepare("SELECT path FROM scan_roots ORDER BY path")?;
        let rows = statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let c = Connection::open(path)?;
        c.execute_batch(include_str!("../schema.sql"))?;
        Self::migrate(&c)?;
        Ok(Self {
            conn: Mutex::new(c),
        })
    }

    /// Cria uma instância em memória (ideal para testes unitários rápidos)
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let c = Connection::open_in_memory()?;
        c.execute_batch(include_str!("../schema.sql"))?;
        Self::migrate(&c)?;
        Ok(Self {
            conn: Mutex::new(c),
        })
    }

    pub fn add_repo(&self, path: &str, url: Option<&str>, mode: &str) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        if let Some(id) = c
            .query_row("SELECT id FROM repos WHERE path=?1", [path], |r| r.get(0))
            .optional()?
        {
            return Ok(id);
        }
        c.execute(
            "INSERT INTO repos(path, url, mode, last_activity) VALUES(?1, ?2, ?3, ?4)",
            rusqlite::params![path, url, mode, unix()],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn list_repos(&self) -> anyhow::Result<Vec<(i64, String, String, String)>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, path, mode, COALESCE(head_sha, '') FROM repos ORDER BY path ASC",
        )?;
        let rows = s
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_repo_views(&self) -> anyhow::Result<Vec<RepoView>> {
        let c = self.conn.lock().unwrap();
        let mut statement = c.prepare(
            "SELECT id, path, mode, COALESCE(head_sha,''), COALESCE(default_branch,''),
                    COALESCE(state,'idle') FROM repos ORDER BY path",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok(RepoView {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    mode: row.get(2)?,
                    head_sha: row.get(3)?,
                    branch: row.get(4)?,
                    state: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_repo_path(&self, repo_id: i64) -> anyhow::Result<String> {
        let c = self.conn.lock().unwrap();
        let path: String = c.query_row("SELECT path FROM repos WHERE id=?1", [repo_id], |r| {
            r.get(0)
        })?;
        Ok(path)
    }

    pub fn update_repo_head(&self, repo_id: i64, head_sha: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "UPDATE repos SET head_sha=?1, last_activity=?2 WHERE id=?3",
            rusqlite::params![head_sha, unix(), repo_id],
        )?;
        Ok(())
    }

    pub fn update_repo_branch(&self, repo_id: i64, branch: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "UPDATE repos SET default_branch=?1, state='ready', last_activity=?2 WHERE id=?3",
            rusqlite::params![branch, unix(), repo_id],
        )?;
        Ok(())
    }

    pub fn enqueue(
        &self,
        kind: &str,
        repo_id: i64,
        payload: &serde_json::Value,
        priority: i64,
    ) -> anyhow::Result<i64> {
        self.enqueue_dependent(kind, repo_id, payload, priority, None)
    }

    pub fn enqueue_dependent(
        &self,
        kind: &str,
        repo_id: i64,
        payload: &serde_json::Value,
        priority: i64,
        depends_on: Option<i64>,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute(
            "INSERT INTO jobs(kind, repo_id, payload_json, priority, created_at, depends_on)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                kind,
                repo_id,
                payload.to_string(),
                priority,
                now,
                depends_on
            ],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn lease_next(&self, ttl_secs: i64) -> anyhow::Result<Option<(i64, String, i64, String)>> {
        let mut c = self.conn.lock().unwrap();
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = unix();
        let row = tx
            .query_row(
                "SELECT j.id, j.kind, j.repo_id, j.payload_json FROM jobs AS j
                 WHERE (j.status='queued' OR (j.status='running' AND j.lease_until <= ?1))
                   AND (j.depends_on IS NULL OR EXISTS (
                     SELECT 1 FROM jobs AS dependency
                     WHERE dependency.id=j.depends_on AND dependency.status='done'
                   ))
                 ORDER BY j.priority ASC, j.created_at ASC, j.id ASC LIMIT 1",
                [now],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;

        if let Some((id, kind, repo, payload)) = row {
            tx.execute(
                "UPDATE jobs SET status='running', lease_until=?1, attempts=attempts+1 WHERE id=?2",
                rusqlite::params![now + ttl_secs, id],
            )?;
            tx.commit()?;
            Ok(Some((id, kind, repo, payload)))
        } else {
            tx.commit()?;
            Ok(None)
        }
    }

    pub fn finish(&self, id: i64, err: Option<&str>) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        let status = if err.is_some() { "failed" } else { "done" };
        c.execute(
            "UPDATE jobs SET status=?1, finished_at=?2, error=?3 WHERE id=?4",
            rusqlite::params![status, unix(), err, id],
        )?;
        Ok(())
    }

    pub fn event(
        &self,
        repo_id: i64,
        kind: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO events(ts, repo_id, kind, payload_json) VALUES(?1, ?2, ?3, ?4)",
            rusqlite::params![unix(), repo_id, kind, payload.to_string()],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn recent_events(&self, limit: usize) -> anyhow::Result<Vec<EventRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, ts, repo_id, kind, payload_json FROM events ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = s
            .query_map([limit], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn events_after(&self, after_id: i64, limit: usize) -> anyhow::Result<Vec<EventRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, ts, repo_id, kind, payload_json FROM events WHERE id > ?1 ORDER BY id ASC LIMIT ?2",
        )?;
        let rows = s
            .query_map(rusqlite::params![after_id, limit], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn recent_jobs(&self, limit: usize) -> anyhow::Result<Vec<JobRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, kind, COALESCE(repo_id, 0), status, attempts FROM jobs ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = s
            .query_map([limit], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_finding(
        &self,
        repo_id: i64,
        blob_sha: &str,
        rule_id: &str,
        severity: &str,
        path: &str,
        line: i64,
        message: &str,
        owasp: &str,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute(
            "INSERT INTO findings(repo_id, blob_sha, rule_id, severity, path, line, message, owasp, first_seen, last_seen)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
             ON CONFLICT(repo_id, blob_sha, rule_id, path, line)
             DO UPDATE SET last_seen=?9",
            rusqlite::params![repo_id, blob_sha, rule_id, severity, path, line, message, owasp, now],
        )?;
        Ok(())
    }

    pub fn list_findings(&self, repo_id: i64) -> anyhow::Result<Vec<FindingRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT rule_id, severity, path, line, message FROM findings WHERE repo_id=?1 ORDER BY line ASC",
        )?;
        let rows = s
            .query_map([repo_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_recipe(
        &self,
        repo_id: i64,
        kind: &str,
        input_hashes: &str,
        policy_hash: &str,
        output_hash: &str,
        plan_json: &str,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO recipes(repo_id, kind, input_hashes, policy_hash, output_hash, plan_json, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![repo_id, kind, input_hashes, policy_hash, output_hash, plan_json, unix()],
        )?;
        Ok(c.last_insert_rowid())
    }

    // ── Migrações leves (idempotentes) ─────────────────────────────────────

    fn migrate(c: &Connection) -> anyhow::Result<()> {
        // F03: coluna de última execução das políticas (cron)
        let _ = c.execute(
            "ALTER TABLE sync_policies ADD COLUMN last_run INTEGER",
            [],
        );
        // gh-ci: dedup de runs de CI (idempotência do ci-scan)
        let _ = c.execute_batch(
            "CREATE TABLE IF NOT EXISTS ci_runs (
               run_id INTEGER PRIMARY KEY,
               repo TEXT NOT NULL,
               workflow TEXT,
               conclusion TEXT,
               seen_at INTEGER NOT NULL
             )",
        );
        Ok(())
    }

    // ── Políticas de sync (F03 cron / F10 path_sync) ────────────────────────

    pub fn add_policy(
        &self,
        kind: &str,
        src_repo: Option<i64>,
        dst_repo: Option<i64>,
        refspec: Option<&str>,
        path_glob: Option<&str>,
        cron_expr: Option<&str>,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO sync_policies(src_repo,dst_repo,kind,refspec,path_glob,cron,enabled)
             VALUES(?1,?2,?3,?4,?5,?6,1)",
            rusqlite::params![src_repo, dst_repo, kind, refspec, path_glob, cron_expr],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn list_policies(&self) -> anyhow::Result<Vec<PolicyRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, src_repo, dst_repo, kind, refspec, path_glob, cron
             FROM sync_policies WHERE enabled=1 ORDER BY id",
        )?;
        let rows = s
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn mark_policy_run(&self, id: i64) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "UPDATE sync_policies SET last_run=?1 WHERE id=?2",
            rusqlite::params![unix(), id],
        )?;
        Ok(())
    }

    // ── Worktrees LRU (F02) ─────────────────────────────────────────────────

    pub fn worktree_get(&self, repo_id: i64, purpose: &str) -> anyhow::Result<Option<String>> {
        let c = self.conn.lock().unwrap();
        let row = c
            .query_row(
                "SELECT path FROM worktrees WHERE repo_id=?1 AND purpose=?2 ORDER BY last_used DESC LIMIT 1",
                rusqlite::params![repo_id, purpose],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        Ok(row)
    }

    pub fn worktree_count(&self) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        Ok(c.query_row("SELECT COUNT(*) FROM worktrees", [], |r| r.get(0))?)
    }

    pub fn worktree_insert(
        &self,
        repo_id: i64,
        label: &str,
        path: &str,
        purpose: &str,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO worktrees(repo_id,branch,path,purpose,last_used,created_at)
             VALUES(?1,?2,?3,?4,?5,?5)
             ON CONFLICT(path) DO UPDATE SET last_used=excluded.last_used",
            rusqlite::params![repo_id, label, path, purpose, lru_clock()],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn worktree_touch(&self, path: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "UPDATE worktrees SET last_used=?1 WHERE path=?2",
            rusqlite::params![lru_clock(), path],
        )?;
        Ok(())
    }

    /// Retorna (path, repo_id) dos worktrees menos recentemente usados (LRU).
    pub fn worktree_lru_victims(&self, excess: i64) -> anyhow::Result<Vec<(String, i64)>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT path, repo_id FROM worktrees ORDER BY last_used ASC LIMIT ?1",
        )?;
        let rows = s
            .query_map([excess], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn worktree_remove(&self, path: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute("DELETE FROM worktrees WHERE path=?1", [path])?;
        Ok(())
    }

    // ── Recipes (F12 exec / F16 replay) ─────────────────────────────────────

    pub fn get_recipe(
        &self,
        id: i64,
    ) -> anyhow::Result<Option<(i64, String, String, String, String)>> {
        let c = self.conn.lock().unwrap();
        let row = c
            .query_row(
                "SELECT repo_id, kind, plan_json, input_hashes, COALESCE(output_hash,'')
                 FROM recipes WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()?;
        Ok(row)
    }

    pub fn update_recipe_output(&self, id: i64, output_hash: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "UPDATE recipes SET output_hash=?1 WHERE id=?2",
            rusqlite::params![output_hash, id],
        )?;
        Ok(())
    }

    // ── Vulnerabilidades OSV (F21/F22) ──────────────────────────────────────

    pub fn upsert_vuln(
        &self,
        id: &str,
        aliases: Option<&str>,
        ecosystem: &str,
        name: Option<&str>,
        ranges_json: &str,
        severity: &str,
        fixed_json: &str,
        modified: i64,
        withdrawn: i64,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO vulns(id,aliases_json,ecosystem,name,ranges_json,severity,fixed_json,modified,withdrawn)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(id) DO UPDATE SET
               aliases_json=excluded.aliases_json, ecosystem=excluded.ecosystem,
               name=excluded.name, ranges_json=excluded.ranges_json,
               severity=excluded.severity, fixed_json=excluded.fixed_json,
               modified=excluded.modified, withdrawn=excluded.withdrawn",
            rusqlite::params![id, aliases, ecosystem, name, ranges_json, severity, fixed_json, modified, withdrawn],
        )?;
        Ok(())
    }

    pub fn vuln_modified(&self, id: &str) -> anyhow::Result<Option<i64>> {
        let c = self.conn.lock().unwrap();
        Ok(c
            .query_row("SELECT modified FROM vulns WHERE id=?1", [id], |r| r.get(0))
            .optional()?)
    }

    pub fn list_vulns(&self) -> anyhow::Result<Vec<VulnRecord>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, COALESCE(aliases_json,''), ecosystem, COALESCE(ranges_json,''),
                    COALESCE(severity,'medium'), COALESCE(fixed_json,''), modified, withdrawn
             FROM vulns",
        )?;
        let rows = s
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// SBOM completo: (repo_id, commit, ecosystem, name, version)
    pub fn list_sbom(&self) -> anyhow::Result<Vec<(i64, String, String, String, String)>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT repo_id, commit_sha, ecosystem, name, version FROM sbom",
        )?;
        let rows = s
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn insert_sbom_entry(
        &self,
        repo_id: i64,
        commit: &str,
        ecosystem: &str,
        name: &str,
        version: &str,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT OR IGNORE INTO sbom(repo_id,commit_sha,ecosystem,name,version,purl,direct)
             VALUES(?1,?2,?3,?4,?5,'',0)",
            rusqlite::params![repo_id, commit, ecosystem, name, version],
        )?;
        Ok(())
    }

    // ── Vault de segredos (F51) ─────────────────────────────────────────────

    /// Persiste um segredo cifrado. `ciphertext` e `nonce` em hex.
    pub fn secret_put(&self, name: &str, ciphertext: &str, nonce: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO secrets(name,ciphertext,nonce,updated_at)
             VALUES(?1,?2,?3,?4)
             ON CONFLICT(name) DO UPDATE SET
               ciphertext=excluded.ciphertext, nonce=excluded.nonce,
               updated_at=excluded.updated_at",
            rusqlite::params![name, ciphertext, nonce, unix()],
        )?;
        Ok(())
    }

    /// Retorna (ciphertext, nonce) em hex, se o segredo existir.
    pub fn secret_get(&self, name: &str) -> anyhow::Result<Option<(String, String)>> {
        let c = self.conn.lock().unwrap();
        let row = c
            .query_row(
                "SELECT ciphertext, nonce FROM secrets WHERE name=?1",
                [name],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        Ok(row)
    }

    // ── Grafo de repositórios (F28) ─────────────────────────────────────────

    pub fn graph_upsert_edge(
        &self,
        src: i64,
        dst: i64,
        weight: f64,
        kind: &str,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO graph_edges(src_repo,dst_repo,weight,kind,updated_at)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(src_repo,dst_repo,kind) DO UPDATE SET
               weight=excluded.weight, updated_at=excluded.updated_at",
            rusqlite::params![src, dst, weight, kind, unix()],
        )?;
        Ok(())
    }

    pub fn graph_edges(&self) -> anyhow::Result<Vec<(i64, i64, f64, String)>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT src_repo, dst_repo, weight, kind FROM graph_edges ORDER BY weight DESC",
        )?;
        let rows = s
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Baselines EWMA/MAD (F30) ────────────────────────────────────────────

    pub fn baseline_get(&self, repo_id: i64, metric: &str) -> anyhow::Result<Option<(f64, f64)>> {
        let c = self.conn.lock().unwrap();
        let row = c
            .query_row(
                "SELECT ewma, mad FROM baselines WHERE repo_id=?1 AND metric=?2",
                rusqlite::params![repo_id, metric],
                |r| Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?)),
            )
            .optional()?;
        Ok(row)
    }

    pub fn baseline_update(
        &self,
        repo_id: i64,
        metric: &str,
        ewma: f64,
        mad: f64,
        n: i64,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO baselines(repo_id,metric,ewma,mad,n,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(repo_id,metric) DO UPDATE SET
               ewma=excluded.ewma, mad=excluded.mad, n=excluded.n,
               updated_at=excluded.updated_at",
            rusqlite::params![repo_id, metric, ewma, mad, n, unix()],
        )?;
        Ok(())
    }

    // ── Doc cards + FTS5 (F26/F27) ──────────────────────────────────────────

    pub fn docs_list(&self, repo_id: i64) -> anyhow::Result<Vec<(String, String)>> {
        let c = self.conn.lock().unwrap();
        let mut s =
            c.prepare("SELECT path, content_hash FROM docs WHERE repo_id=?1")?;
        let rows = s
            .query_map([repo_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn doc_upsert(
        &self,
        repo_id: i64,
        path: &str,
        title: &str,
        content: &str,
        content_hash: &str,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO docs(repo_id,path,title,content,content_hash,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(repo_id,path) DO UPDATE SET
               title=excluded.title, content=excluded.content,
               content_hash=excluded.content_hash, updated_at=excluded.updated_at",
            rusqlite::params![repo_id, path, title, content, content_hash, unix()],
        )?;
        let key = format!("{repo_id}:{path}");
        c.execute("DELETE FROM doc_fts WHERE repo_path=?1", [&key])?;
        c.execute(
            "INSERT INTO doc_fts(repo_path,title,content) VALUES(?1,?2,?3)",
            rusqlite::params![key, title, content],
        )?;
        Ok(())
    }

    pub fn doc_delete(&self, repo_id: i64, path: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "DELETE FROM docs WHERE repo_id=?1 AND path=?2",
            rusqlite::params![repo_id, path],
        )?;
        c.execute(
            "DELETE FROM doc_fts WHERE repo_path=?1",
            [format!("{repo_id}:{path}")],
        )?;
        Ok(())
    }

    /// Busca full-text nos doc cards. `query` é uma frase livre; cada termo
    /// é envolvido em aspas para neutralizar a sintaxe do FTS5.
    pub fn doc_search(
        &self,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<(i64, String, String, String)>> {
        let c = self.conn.lock().unwrap();
        let fts_ok = c
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='doc_fts'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !fts_ok {
            return Ok(Vec::new());
        }
        let escaped = query
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| format!("\"{}\"", t.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        if escaped.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.min(100);
        let sql = format!(
            "SELECT d.repo_id, d.path, d.title,
                    snippet(doc_fts, 2, '[', ']', '…', 12) AS snip
             FROM doc_fts JOIN docs d ON d.repo_id || ':' || d.path = doc_fts.repo_path
             WHERE doc_fts MATCH '{escaped}'
             LIMIT {limit}"
        );
        let mut s = c.prepare(&sql)?;
        let rows = s
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Rate budget por forge (F32/F34) ─────────────────────────────────────

    /// Consome 1 unidade do orçamento da janela (hora). Devolve false se
    /// o orçamento `cap` estourou.
    pub fn rate_consume(&self, forge: &str, cap: i64) -> anyhow::Result<bool> {
        let c = self.conn.lock().unwrap();
        let window = unix() / 3600;
        c.execute(
            "INSERT INTO rate_budget(forge,window_start,used)
             VALUES(?1,?2,1)
             ON CONFLICT(forge,window_start) DO UPDATE SET used=used+1",
            rusqlite::params![forge, window],
        )?;
        let used: i64 = c.query_row(
            "SELECT used FROM rate_budget WHERE forge=?1 AND window_start=?2",
            rusqlite::params![forge, window],
            |r| r.get(0),
        )?;
        Ok(used <= cap)
    }

    // ── Entregas de hooks externos (F38) ────────────────────────────────────

    pub fn hook_delivery_insert(
        &self,
        hook: &str,
        event: &str,
        status: &str,
        detail: &str,
    ) -> anyhow::Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO hook_deliveries(hook,event,status,detail,ts)
             VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![hook, event, status, detail, unix()],
        )?;
        Ok(c.last_insert_rowid())
    }

    pub fn hook_deliveries(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<(String, String, String, String)>> {
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare(
            "SELECT hook, event, status, COALESCE(detail,'')
             FROM hook_deliveries ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = s
            .query_map([limit], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── gh-ci: dedup determinístico de runs (idempotência do ci-scan) ───────

    /// `true` se o run de CI já foi relatado por uma passada anterior.
    pub fn ci_seen(&self, run_id: i64) -> anyhow::Result<bool> {
        let c = self.conn.lock().unwrap();
        let n: i64 = c.query_row(
            "SELECT COUNT(*) FROM ci_runs WHERE run_id=?1",
            [run_id],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    /// Marca um run como visto. Idempotente: INSERT OR IGNORE.
    pub fn ci_mark(
        &self,
        run_id: i64,
        repo: &str,
        workflow: &str,
        conclusion: &str,
    ) -> anyhow::Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT OR IGNORE INTO ci_runs(run_id,repo,workflow,conclusion,seen_at)
             VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![run_id, repo, workflow, conclusion, unix()],
        )?;
        Ok(())
    }
}
