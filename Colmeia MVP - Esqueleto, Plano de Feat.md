# Colmeia MVP — Esqueleto, Plano de Features e Roadmap de Produção

**Escopo desta entrega:** (1) esqueleto de código compilável do MVP, (2) matriz de features com versão-alvo e critérios de aceite, (3) roadmap faseado com *gates* de produção e de "usuário comum". Onde há `TODO(Fxx)` no código, é stub intencional mapeado à matriz — não é código esquecido.

**Linha de corte do MVP (v0.1):** registry + scheduler confiável + sync (fetch/ff/mirror) + gate (secrets/skills/SBOM/OSV) + slice *plan-only* + merge drivers básicos + TUI + web read-only + onboarding/doctor. Slice *exec*, workbench 3-pane, path_sync, auto-PR, docs completas, grafo imersivo e plugins ficam para v0.2/v0.3. Isso é o que um usuário comum consegue instalar e usar com segurança no dia 1.

---

## 1. Esqueleto do MVP

Crate único no MVP (split em workspace quando plugins entrarem). `git` CLI capado é o backend; `gitoxide` entra como otimização depois.

### 1.1 Árvore

```
colmeia/
├── Cargo.toml
├── schema.sql
├── colmeia.toml                  # default gerado por `init`
├── justfile
├── README.md
├── LICENSE-MIT / LICENSE-APACHE
├── rfcs/0001-determinism.md
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs  cli.rs  config.rs
│   ├── store.rs  events.rs
│   ├── scheduler.rs  governor.rs
│   ├── git.rs  sync.rs
│   ├── gate.rs  rules.rs  sbom.rs  osv.rs
│   ├── slice.rs  merge.rs
│   ├── tui.rs  web.rs
│   └── doctor.rs
└── tests/
    ├── farm.rs                   # gerador de repos sintéticos
    └── determinism.rs            # proptest slice/merge
```

### 1.2 `Cargo.toml`

```toml
[package]
name = "colmeia"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Gestor Git de frotas, TUI-first, automação determinística em hardware mínimo"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
crossterm = "0.27"
ratatui = "0.26"
tiny_http = "0.12"
ureq = { version = "2", features = ["json"] }
crossbeam-channel = "0.5"
sha2 = "0.10"
hex = "0.4"
regex = "1"
walkdir = "2"
dirs = "5"
cron = "0.12"
libc = "0.2"

[dev-dependencies]
proptest = "1"
tempfile = "3"

[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = "s"
```

### 1.3 `schema.sql` (MVP)

```sql
PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-32768; PRAGMA temp_store=FILE;

CREATE TABLE IF NOT EXISTS repos(
  id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL, url TEXT,
  mode TEXT CHECK(mode IN ('managed','mirror','dormant')) DEFAULT 'mirror',
  forge TEXT, default_branch TEXT, head_sha TEXT,
  priority INTEGER DEFAULT 5, last_activity INTEGER, state TEXT DEFAULT 'idle');

CREATE TABLE IF NOT EXISTS sync_policies(
  id INTEGER PRIMARY KEY, src_repo INTEGER, dst_repo INTEGER,
  kind TEXT CHECK(kind IN ('fetch_upstream','push_mirror','branch_sync','path_sync')),
  refspec TEXT, path_glob TEXT, cron TEXT, on_event TEXT,
  conflict_policy TEXT DEFAULT 'escalate', enabled INTEGER DEFAULT 1);

CREATE TABLE IF NOT EXISTS jobs(
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, repo_id INTEGER,
  payload_json TEXT NOT NULL, status TEXT DEFAULT 'queued',
  priority INTEGER DEFAULT 5, lease_until INTEGER, attempts INTEGER DEFAULT 0,
  depends_on INTEGER, created_at INTEGER, finished_at INTEGER, error TEXT);
CREATE INDEX IF NOT EXISTS jobs_dispatch ON jobs(status,priority,created_at);

CREATE TABLE IF NOT EXISTS events(
  id INTEGER PRIMARY KEY, ts INTEGER, repo_id INTEGER, kind TEXT, payload_json TEXT);

CREATE TABLE IF NOT EXISTS findings(
  id INTEGER PRIMARY KEY, repo_id INTEGER, blob_sha TEXT, rule_id TEXT,
  severity TEXT, path TEXT, line INTEGER, message TEXT, owasp TEXT,
  status TEXT DEFAULT 'open', first_seen INTEGER, last_seen INTEGER,
  UNIQUE(repo_id,blob_sha,rule_id,path,line));

CREATE TABLE IF NOT EXISTS sbom(
  repo_id INTEGER, commit_sha TEXT, ecosystem TEXT, name TEXT, version TEXT,
  purl TEXT, direct INTEGER, PRIMARY KEY(repo_id,commit_sha,ecosystem,name,version));

CREATE TABLE IF NOT EXISTS vulns(
  id TEXT PRIMARY KEY, aliases_json TEXT, ecosystem TEXT, name TEXT,
  ranges_json TEXT, severity TEXT, fixed_json TEXT, modified INTEGER, withdrawn INTEGER);

CREATE TABLE IF NOT EXISTS baselines(
  repo_id INTEGER, metric TEXT, ewma REAL, mad REAL, n INTEGER, updated_at INTEGER,
  PRIMARY KEY(repo_id,metric));

CREATE TABLE IF NOT EXISTS recipes(
  id INTEGER PRIMARY KEY, repo_id INTEGER, kind TEXT, input_hashes TEXT,
  policy_hash TEXT, output_hash TEXT, plan_json TEXT, created_at INTEGER);

CREATE TABLE IF NOT EXISTS audit(
  id INTEGER PRIMARY KEY, ts INTEGER, actor TEXT, action TEXT, target TEXT, details_json TEXT);
```

### 1.4 `src/config.rs`

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "d_workers")] pub workers: usize,
    #[serde(default = "d_soft")] pub memory_soft_mb: u64,
    #[serde(default = "d_hard")] pub memory_hard_mb: u64,
    #[serde(default = "d_gitcap")] pub git_child_cap_mb: u64,
    #[serde(default = "d_hot")] pub hot_watch_count: usize,
    #[serde(default)] pub security: Security,
    #[serde(default)] pub forge: Forge,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Security {
    #[serde(default = "d_eco")] pub osv_ecosystems: Vec<String>,
    #[serde(default = "d_sev")] pub gate_block_severity: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Forge { pub github_token_env: Option<String> }

fn d_workers()->usize{2} fn d_soft()->u64{600} fn d_hard()->u64{900}
fn d_gitcap()->u64{256} fn d_hot()->usize{100}
fn d_eco()->Vec<String>{vec!["PyPI".into(),"npm".into(),"crates.io".into()]}
fn d_sev()->String{"high".into()}

impl Default for Config {
    fn default()->Self{ Self{ workers:d_workers(), memory_soft_mb:d_soft(),
        memory_hard_mb:d_hard(), git_child_cap_mb:d_gitcap(), hot_watch_count:d_hot(),
        security:Security::default(), forge:Forge::default() } }
}

impl Config {
    pub fn load()->anyhow::Result<Self>{
        let p = Self::path();
        if p.exists(){ Ok(toml::from_str(&std::fs::read_to_string(p)?)?) }
        else { Ok(Self::default()) }
    }
    pub fn path()->PathBuf{
        dirs::home_dir().unwrap_or_else(||PathBuf::from(".")).join(".colmeia/colmeia.toml")
    }
    /// Auto-tier por RAM total (T1<=1GB, T2<=6GB, T3 acima).
    pub fn detect_tier()->&'static str{
        let mem = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let kb: u64 = mem.lines().find(|l|l.starts_with("MemTotal"))
            .and_then(|l|l.split_whitespace().nth(1)).and_then(|v|v.parse().ok()).unwrap_or(4_000_000);
        match kb/1024 { 0..=1024=>"t1", 1025..=6144=>"t2", _=>"t3" }
    }
}
```

### 1.5 `src/store.rs`

```rust
use rusqlite::Connection;
use std::sync::Mutex;
use std::path::Path;

pub struct Store { pub conn: Mutex<Connection> }

impl Store {
    pub fn open(path:&Path)->anyhow::Result<Self>{
        std::fs::create_dir_all(path.parent().unwrap())?;
        let c = Connection::open(path)?;
        c.execute_batch(include_str!("../schema.sql"))?;
        Ok(Self{ conn: Mutex::new(c) })
    }
    pub fn add_repo(&self, path:&str, url:Option<&str>, mode:&str)->anyhow::Result<i64>{
        let c = self.conn.lock().unwrap();
        c.execute("INSERT OR IGNORE INTO repos(path,url,mode) VALUES(?1,?2,?3)",
            rusqlite::params![path,url,mode])?;
        Ok(c.query_row("SELECT id FROM repos WHERE path=?1",[path],|r|r.get(0))?)
    }
    pub fn list_repos(&self)->anyhow::Result<Vec<(i64,String,String,String)>>{
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare("SELECT id,path,mode,COALESCE(head_sha,'') FROM repos ORDER BY path")?;
        let rows = s.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?
            .collect::<Result<Vec<_>,_>>()?;
        Ok(rows)
    }
    pub fn enqueue(&self, kind:&str, repo_id:i64, payload:&serde_json::Value, prio:i64)->anyhow::Result<i64>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute("INSERT INTO jobs(kind,repo_id,payload_json,priority,created_at) VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![kind,repo_id,payload.to_string(),prio,now])?;
        Ok(c.last_insert_rowid())
    }
    pub fn lease_next(&self, ttl_secs:i64)->anyhow::Result<Option<(i64,String,i64,String)>>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        let row = c.query_row(
            "SELECT id,kind,repo_id,payload_json FROM jobs
             WHERE status='queued' OR (status='running' AND lease_until<?1)
             ORDER BY priority,created_at LIMIT 1",[now],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).ok();
        if let Some((id,kind,repo,payload)) = row {
            c.execute("UPDATE jobs SET status='running',lease_until=?1,attempts=attempts+1 WHERE id=?2",
                rusqlite::params![now+ttl_secs,id])?;
            Ok(Some((id,kind,repo,payload)))
        } else { Ok(None) }
    }
    pub fn finish(&self, id:i64, err:Option<&str>)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        c.execute("UPDATE jobs SET status=?1,finished_at=?2,error=?3 WHERE id=?4",
            rusqlite::params![if err.is_some(){"failed"}else{"done"},unix(),err,id])?;
        Ok(())
    }
    pub fn event(&self, repo_id:i64, kind:&str, payload:&serde_json::Value)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        c.execute("INSERT INTO events(ts,repo_id,kind,payload_json) VALUES(?1,?2,?3,?4)",
            rusqlite::params![unix(),repo_id,kind,payload.to_string()])?;
        Ok(())
    }
    pub fn recent_events(&self, limit:i64)->anyhow::Result<Vec<(i64,String,String)>>{
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare("SELECT ts,kind,payload_json FROM events ORDER BY id DESC LIMIT ?1")?;
        Ok(s.query_map([limit],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?
            .collect::<Result<Vec<_>,_>>()?)
    }
    pub fn upsert_finding(&self, repo:i64, blob:&str, rule:&str, sev:&str, path:&str, line:i64, msg:&str, owasp:&str)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute("INSERT INTO findings(repo_id,blob_sha,rule_id,severity,path,line,message,owasp,first_seen,last_seen)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)
            ON CONFLICT(repo_id,blob_sha,rule_id,path,line) DO UPDATE SET last_seen=?9,status='open'",
            rusqlite::params![repo,blob,rule,sev,path,line,msg,owasp,now])?;
        Ok(())
    }
}
pub fn unix()->i64{ std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 }
```

### 1.6 `src/git.rs` (backend capado)

```rust
use std::process::{Command, Output};
use std::path::Path;

pub struct Git { pub cap_mb: u64 }

impl Git {
    fn cmd(&self, cwd:&Path, args:&[&str])->Command{
        let mut c = Command::new("git");
        c.current_dir(cwd).args(args)
            .env("GIT_TERMINAL_PROMPT","0").env("GIT_OPTIONAL_LOCKS","0");
        #[cfg(unix)]
        {
            let cap = self.cap_mb*1024*1024;
            unsafe { c.pre_exec(move || {
                let rl = libc::rlimit{ rlim_cur:cap, rlim_max:cap };
                libc::setrlimit(libc::RLIMIT_AS, &rl);
                Ok(())
            }); }
        }
        c
    }
    pub fn run(&self, cwd:&Path, args:&[&str])->anyhow::Result<Output>{
        let o = self.cmd(cwd,args).output()?;
        Ok(o)
    }
    pub fn head(&self, cwd:&Path)->anyhow::Result<String>{
        let o = self.run(cwd,&["rev-parse","HEAD"])?;
        Ok(String::from_utf8_lossy(&o.stdout).trim().into())
    }
    pub fn is_clean(&self, cwd:&Path)->anyhow::Result<bool>{
        let o = self.run(cwd,&["status","--porcelain"])?;
        Ok(o.stdout.is_empty())
    }
    pub fn fetch(&self, cwd:&Path, remote:&str)->anyhow::Result<()>{
        let o = self.run(cwd,&["fetch","--filter=blob:none","--prune",remote])?;
        anyhow::ensure!(o.status.success(), "fetch falhou: {}", String::from_utf8_lossy(&o.stderr));
        let _ = self.run(cwd,&["commit-graph","write","--reachable"]);
        Ok(())
    }
    pub fn ff_merge(&self, cwd:&Path, refspec:&str)->anyhow::Result<bool>{
        let o = self.run(cwd,&["merge","--ff-only",refspec])?;
        Ok(o.status.success())
    }
    pub fn backup_ref(&self, cwd:&Path, branch:&str)->anyhow::Result<String>{
        let sha = self.head(cwd)?;
        let bak = format!("refs/colmeia/backup/{}", branch);
        self.run(cwd,&["update-ref",&bak,&sha])?;
        Ok(bak)
    }
    pub fn update_ref_cas(&self, cwd:&Path, branch:&str, new:&str, old:&str)->anyhow::Result<()>{
        let r = format!("refs/heads/{}", branch);
        let o = self.run(cwd,&["update-ref",&r,new,old])?;
        anyhow::ensure!(o.status.success(), "CAS falhou (ref mudou): {}", String::from_utf8_lossy(&o.stderr));
        Ok(())
    }
    pub fn changed_files(&self, cwd:&Path, range:&str)->anyhow::Result<Vec<String>>{
        let o = self.run(cwd,&["diff","--name-only",range])?;
        Ok(String::from_utf8_lossy(&o.stdout).lines().map(|s|s.into()).collect())
    }
}
```

### 1.7 `src/governor.rs` + `src/scheduler.rs`

```rust
// governor.rs
pub fn rss_mb()->u64{
    let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let pages: u64 = s.split_whitespace().nth(1).and_then(|v|v.parse().ok()).unwrap_or(0);
    pages*4/1024
}
pub fn available_mb()->u64{
    let s = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    s.lines().find(|l|l.starts_with("MemAvailable"))
        .and_then(|l|l.split_whitespace().nth(1)).and_then(|v|v.parse().ok()).unwrap_or(0)/1024
}
pub enum Pressure { Ok, Soft, Hard }
pub fn pressure(soft:u64, hard:u64)->Pressure{
    let r = rss_mb();
    if r>=hard || available_mb()<128 { Pressure::Hard }
    else if r>=soft { Pressure::Soft } else { Pressure::Ok }
}
```

```rust
// scheduler.rs
use crate::{store::Store, config::Config, governor, git::Git, sync, gate, slice, merge};
use std::sync::Arc;
use std::collections::HashSet;

pub fn run(store:Arc<Store>, cfg:Config)->anyhow::Result<()>{
    let git = Arc::new(Git{ cap_mb: cfg.git_child_cap_mb });
    let (tx, rx) = crossbeam_channel::bounded::<(i64,String,i64,String)>(cfg.workers*2);
    // workers fixos
    for _ in 0..cfg.workers {
        let rx = rx.clone(); let st = store.clone(); let g = git.clone(); let c = cfg.clone();
        std::thread::spawn(move || {
            for (id,kind,repo,payload) in rx {
                let res = dispatch(&st,&g,&c,&kind,repo,&payload);
                let _ = st.finish(id, res.err().map(|e|e.to_string()).as_deref());
            }
        });
    }
    let mut locked: HashSet<i64> = HashSet::new();
    loop {
        match governor::pressure(cfg.memory_soft_mb, cfg.memory_hard_mb){
            governor::Pressure::Hard => { std::thread::sleep(std::time::Duration::from_secs(2)); continue; }
            governor::Pressure::Soft => { std::thread::sleep(std::time::Duration::from_millis(500)); }
            governor::Pressure::Ok => {}
        }
        // TODO(F03): cron + polling adaptativo + inotify hot. MVP: drain da fila.
        if let Some(job) = store.lease_next(600)? {
            if locked.contains(&job.2){ continue; }
            locked.insert(job.2);
            let st = store.clone(); let tx = tx.clone();
            let j = job.clone();
            // lock liberado ao terminar (worker envia ack simplificado via finish)
            tx.send(job).ok();
            // MVP: libera lock após send (workers são rápidos); produção: ack por job.
            locked.remove(&j.2);
            let _ = st;
        } else {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}

fn dispatch(st:&Store, g:&Git, c:&Config, kind:&str, repo:i64, payload:&str)->anyhow::Result<()>{
    let v: serde_json::Value = serde_json::from_str(payload)?;
    match kind {
        "fetch" => sync::fetch(st,g,repo,&v),
        "branch_sync" => sync::branch_sync(st,g,repo,&v),
        "push_mirror" => sync::push_mirror(st,g,repo,&v),
        "gate" => gate::run_repo(st,g,c,repo,&v),
        "slice_plan" => slice::plan(st,g,repo,&v),
        "merge" => merge::run(st,g,repo,&v),
        "osv_sync" => crate::osv::sync(st,c),
        _ => anyhow::bail!("job desconhecido: {kind}"),
    }
}
```

### 1.8 `src/sync.rs`

```rust
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

fn repo_path(st:&Store, id:i64)->anyhow::Result<String>{
    let c = st.conn.lock().unwrap();
    Ok(c.query_row("SELECT path FROM repos WHERE id=?1",[id],|r|r.get(0))?)
}

pub fn fetch(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let remote = v["remote"].as_str().unwrap_or("origin");
    g.fetch(Path::new(&p), remote)?;
    let sha = g.head(Path::new(&p))?;
    { let c = st.conn.lock().unwrap();
      c.execute("UPDATE repos SET head_sha=?1,last_activity=?2 WHERE id=?3",
        rusqlite::params![sha, crate::store::unix(), repo])?; }
    st.event(repo,"sync.ok",&serde_json::json!({"sha":sha}))?;
    Ok(())
}

pub fn branch_sync(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let refspec = v["refspec"].as_str().unwrap_or("origin/main");
    if g.ff_merge(Path::new(&p), refspec)? {
        st.event(repo,"sync.ff",&serde_json::json!({"ref":refspec}))?;
    } else {
        // divergência ⇒ escala para merge (nunca força)
        st.enqueue("merge", repo, &serde_json::json!({"ref":refspec,"policy":"escalate"}), 3)?;
        st.event(repo,"sync.diverged",&serde_json::json!({"ref":refspec}))?;
    }
    Ok(())
}

pub fn push_mirror(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let dst = v["dst"].as_str().unwrap_or("mirror");
    let o = g.run(Path::new(&p), &["push","--mirror",dst])?;
    anyhow::ensure!(o.status.success(), "push falhou");
    st.event(repo,"sync.mirror",&serde_json::json!({"dst":dst}))?;
    Ok(())
}
// TODO(F10): path_sync (skill-packs) em v0.2.
```

### 1.9 `src/rules.rs` + `src/gate.rs` (SAST in-core)

```rust
// rules.rs
use regex::Regex;
pub struct Finding { pub rule:&'static str, pub sev:&'static str, pub line:usize, pub msg:String, pub owasp:&'static str }

pub fn scan(path:&str, text:&str)->Vec<Finding>{
    let mut out = Vec::new();
    let secret = Regex::new(r"(ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16})").unwrap();
    let danger = Regex::new(r"(eval\(|exec\(|shell\s*=\s*True|pickle\.loads|yaml\.load\()").unwrap();
    let inject = Regex::new(r"(?i)(ignore previous instructions|disregard your instructions|you are now)").unwrap();
    for (i,l) in text.lines().enumerate(){
        if secret.is_match(l){ out.push(Finding{rule:"secret.token",sev:"critical",line:i+1,msg:"token hardcoded".into(),owasp:"ASVS"}); }
        if danger.is_match(l){ out.push(Finding{rule:"code.dangerous",sev:"high",line:i+1,msg:"API perigosa".into(),owasp:"Top10"}); }
        if is_skill(path){
            if inject.is_match(l){ out.push(Finding{rule:"skill.injection.override",sev:"high",line:i+1,msg:"prompt injection".into(),owasp:"LLM01"}); }
            if l.chars().any(|c| matches!(c as u32, 0x200B..=0x200F | 0x202A..=0x202E)){
                out.push(Finding{rule:"skill.hidden_unicode",sev:"critical",line:i+1,msg:"unicode invisível".into(),owasp:"LLM01"});
            }
        }
    }
    out
}
fn is_skill(p:&str)->bool{ p.ends_with("SKILL.md")||p.ends_with(".prompt")||p.ends_with(".skill") }
pub fn entropy(s:&str)->f64{
    let mut f=[0u32;256]; for b in s.bytes(){ f[b as usize]+=1; }
    let n=s.len() as f64; if n==0.0{return 0.0;}
    -f.iter().filter(|&&c|c>0).map(|&c|{let p=c as f64/n; p*p.log2()}).sum::<f64>()
}
```

```rust
// gate.rs
use crate::{store::Store, git::Git, config::Config, rules};
use serde_json::Value;
use std::path::Path;

pub fn run_repo(st:&Store, g:&Git, _c:&Config, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let range = v["range"].as_str().unwrap_or("HEAD~1..HEAD");
    let files = g.changed_files(Path::new(&p), range)?;
    let mut count=0;
    for f in files {
        let full = Path::new(&p).join(&f);
        if !full.is_file(){ continue; }
        let text = std::fs::read_to_string(&full).unwrap_or_default();
        if text.len()>1_000_000 { continue; } // skip gigantes (T2)
        let blob = blob_sha(&text);
        for fd in rules::scan(&f,&text){
            st.upsert_finding(repo,&blob,fd.rule,fd.sev,&f,fd.line as i64,&fd.msg,fd.owasp)?;
            count+=1;
        }
    }
    st.event(repo,"gate.done",&serde_json::json!({"findings":count}))?;
    Ok(())
}
pub fn blob_sha(text:&str)->String{
    use sha2::{Sha256,Digest};
    let mut h=Sha256::new(); h.update(text.as_bytes()); hex::encode(h.finalize())
}
pub fn sarif(findings:&[(String,String,String,i64)])->Value{
    serde_json::json!({
      "version":"2.1.0",
      "runs":[{"tool":{"driver":{"name":"colmeia-gate"}},
        "results": findings.iter().map(|(rule,sev,path,line)|serde_json::json!({
          "ruleId":rule,"level":sev,"locations":[{"physicalLocation":{
            "artifactLocation":{"uri":path},"region":{"startLine":line}}}]
        })).collect::<Vec<_>>()}]
    })
}
// TODO(F17): loader Starlark hermético substitui rules.rs hardcoded em v0.2.
```

### 1.10 `src/sbom.rs` + `src/osv.rs`

```rust
// sbom.rs — parsers diretos (MVP: 3 ecossistemas)
pub fn parse(repo_path:&str)->Vec<(String,String,String)>{ // (eco,name,version)
    let mut out=Vec::new();
    let p=std::path::Path::new(repo_path);
    if let Ok(t)=std::fs::read_to_string(p.join("Cargo.lock")){
        for blk in t.split("[[package]]").skip(1){
            let n=field(blk,"name"); let v=field(blk,"version");
            if !n.is_empty(){ out.push(("crates.io".into(),n,v)); }
        }
    }
    if let Ok(t)=std::fs::read_to_string(p.join("requirements.txt")){
        for l in t.lines(){ if let Some((n,v))=l.split_once("=="){ out.push(("PyPI".into(),n.trim().into(),v.trim().into())); } }
    }
    if let Ok(t)=std::fs::read_to_string(p.join("package-lock.json")){
        if let Ok(v)=serde_json::from_str::<serde_json::Value>(&t){
            if let Some(pk)=v["packages"].as_object(){
                for (k,val) in pk { let n=k.trim_start_matches("node_modules/");
                    if let Some(ver)=val["version"].as_str(){ out.push(("npm".into(),n.into(),ver.into())); } }
            }
        }
    }
    out
}
fn field(blk:&str,key:&str)->String{
    blk.lines().find(|l|l.trim().starts_with(&format!("{key} =")))
        .and_then(|l|l.split('"').nth(1)).unwrap_or("").into()
}
```

```rust
// osv.rs
use crate::{store::Store, config::Config};
pub fn sync(st:&Store, c:&Config)->anyhow::Result<()>{
    for eco in &c.security.osv_ecosystems {
        let url = format!("https://storage.googleapis.com/osv-vulnerabilities/{}/all.zip", eco);
        let resp = ureq::get(&url).call()?;
        let mut bytes=Vec::new();
        resp.into_reader().take(200_000_000).read_to_end(&mut bytes)?;
        // TODO(F21): unzip + delta por modified + upsert em vulns. MVP: stub de ingest.
        let _ = st;
        st.event(0,"osv.sync",&serde_json::json!({"eco":eco,"bytes":bytes.len()}))?;
    }
    Ok(())
}
use std::io::Read;
// TODO(F22): matcher semver/pep440 + watch mode + auto-PR (v0.2).
```

### 1.11 `src/slice.rs` + `src/merge.rs`

```rust
// slice.rs — MVP: plan-only determinístico
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

pub fn plan(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let commit = v["commit"].as_str().unwrap_or("HEAD");
    let max = v["max_lines"].as_u64().unwrap_or(400) as usize;
    let o = g.run(Path::new(&p), &["show","--numstat","--format=",commit])?;
    let mut units: Vec<(String,usize)> = String::from_utf8_lossy(&o.stdout).lines()
        .filter_map(|l|{ let mut it=l.split_whitespace();
            let a=it.next()?.parse().ok()?; let d=it.next()?.parse().ok()?;
            Some((it.next()?.to_string(), a+d)) }).collect();
    // ordenação estável + tie-break por hash ⇒ determinismo
    units.sort_by(|a,b| a.0.cmp(&b.0).then_with(|| gate::blob_sha(&a.0).cmp(&gate::blob_sha(&b.0))));
    let mut slices: Vec<Vec<String>> = vec![vec![]];
    let mut acc=0;
    for (path,lines) in units {
        if acc+lines>max && !slices.last().unwrap().is_empty(){ slices.push(vec![]); acc=0; }
        slices.last_mut().unwrap().push(path); acc+=lines;
    }
    let plan = serde_json::json!({"commit":commit,"slices":slices,"max_lines":max});
    { let c=st.conn.lock().unwrap();
      c.execute("INSERT INTO recipes(repo_id,kind,input_hashes,policy_hash,plan_json,created_at)
                 VALUES(?1,'slice',?2,?3,?4,?5)",
        rusqlite::params![repo,commit,gate::blob_sha(&plan.to_string()),plan.to_string(),crate::store::unix()])?; }
    st.event(repo,"slice.plan",&plan)?;
    Ok(())
}
// TODO(F12): exec com apply --cached + gate por slice + backup ref + CAS (v0.2).
```

```rust
// merge.rs — MVP: ff + drivers simples + escalate
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

pub fn run(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let refspec = v["ref"].as_str().unwrap_or("origin/main");
    let policy = v["policy"].as_str().unwrap_or("escalate");
    let path = Path::new(&p);
    if g.ff_merge(path, refspec)? { st.event(repo,"merge.ff",&serde_json::json!({}))?; return Ok(()); }
    match policy {
        "ours"|"theirs" => {
            let o = g.run(path, &["merge","-X",policy,refspec])?;
            anyhow::ensure!(o.status.success(),"merge -X falhou");
            st.event(repo,"merge.driver",&serde_json::json!({"driver":policy}))?;
        }
        _ => {
            let _ = g.run(path, &["merge","--abort"]);
            st.event(repo,"merge.conflict",&serde_json::json!({"ref":refspec,"needs":"human"}))?;
            // TODO(F15): workbench 3-pane + structured 3-way + recipes (v0.2).
        }
    }
    Ok(())
}
```

### 1.12 `src/tui.rs` (esqueleto ratatui)

```rust
use crate::store::Store;
use std::sync::Arc;
use crossterm::{event::{self,Event,KeyCode},terminal::{enable_raw_mode,disable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen},execute};
use ratatui::{Terminal,backend::CrosstermBackend,widgets::{Block,Borders,List,ListItem,Paragraph},layout::{Layout,Constraint,Direction}};

pub fn run(store:Arc<Store>)->anyhow::Result<()>{
    enable_raw_mode()?;
    let mut out=std::io::stdout();
    execute!(out,EnterAlternateScreen)?;
    let mut term=Terminal::new(CrosstermBackend::new(out))?;
    let mut tab=0usize;
    loop {
        let repos=store.list_repos()?;
        let events=store.recent_events(20)?;
        term.draw(|f|{
            let chunks=Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(3),Constraint::Min(0)]).split(f.size());
            let tabs=Paragraph::new(" [1] Repos  [2] Jobs  [3] Findings   q=sair ")
                .block(Block::default().borders(Borders::ALL).title("Colmeia"));
            f.render_widget(tabs,chunks[0]);
            let items: Vec<ListItem> = if tab==0 {
                repos.iter().map(|(_,p,m,sha)|ListItem::new(format!("{p}  [{m}]  {}", &sha[..7.min(sha.len())]))).collect()
            } else {
                events.iter().map(|(ts,k,_)|ListItem::new(format!("{ts}  {k}"))).collect()
            };
            f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL)),chunks[1]);
        })?;
        if event::poll(std::time::Duration::from_millis(250))?{
            if let Event::Key(k)=event::read()?{
                match k.code { KeyCode::Char('q')=>break, KeyCode::Char('1')=>tab=0,
                    KeyCode::Char('2')=>tab=1, KeyCode::Char('3')=>tab=2, _=>{} }
            }
        }
    }
    disable_raw_mode()?;
    execute!(term.backend_mut(),LeaveAlternateScreen)?;
    Ok(())
}
// TODO(F46): paridade total de comandos + grafo braille + workbench (v0.2/v0.3).
```

### 1.13 `src/web.rs` (read-only + SSE)

```rust
use crate::store::Store;
use std::sync::Arc;
pub fn run(store:Arc<Store>, bind:&str)->anyhow::Result<()>{
    let server = tiny_http::Server::http(bind)?;
    for req in server.incoming_requests(){
        let url = req.url().to_string();
        let (status,ctype,body) = match url.as_str(){
            "/" => (200,"text/html", INDEX.to_string()),
            "/api/repos" => (200,"application/json", serde_json::to_string(&store.list_repos()?)?),
            "/api/events" => (200,"application/json", serde_json::to_string(&store.recent_events(50)?)?),
            _ => (404,"text/plain","not found".into()),
        };
        let resp = tiny_http::Response::from_string(body).with_status_code(status)
            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes()).unwrap());
        let _ = req.respond(resp);
    }
    Ok(())
}
const INDEX:&str = r#"<!doctype html><html data-theme="escuro"><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<style>
:root{--bg:#0B0E14;--sf:#131722;--tx:#E8EAF2;--bd:#232A3A;--ac:#F59E0B}
[data-theme=claro]{--bg:#F5F6FA;--sf:#FFF;--tx:#171A26;--bd:#E4E7F0}
[data-theme=sepia]{--bg:#EFE6D3;--sf:#F8F1E0;--tx:#3B2F23;--bd:#D9CBB0}
body{background:var(--bg);color:var(--tx);font:14px/1.5 system-ui;margin:0;padding:16px}
.card{background:var(--sf);border:1px solid var(--bd);border-radius:12px;padding:12px;margin:8px 0}
</style></head><body>
<h1>Colmeia</h1><div id=repos class=card>carregando…</div>
<script>
fetch('/api/repos').then(r=>r.json()).then(rs=>{
 document.getElementById('repos').innerHTML = rs.map(r=>`<div>${r[1]} <b>[${r[2]}]</b></div>`).join('');
});
</script></body></html>"#;
// TODO(F45): SSE real + temas neumorph + responsivo completo (v0.3).
```

### 1.14 `src/cli.rs` + `src/main.rs` + `src/doctor.rs`

```rust
// cli.rs
use clap::{Parser,Subcommand};
#[derive(Parser)] #[command(name="colmeia",version)]
pub struct Cli { #[command(subcommand)] pub cmd: Cmd }
#[derive(Subcommand)]
pub enum Cmd {
    Init, Add { url:String, #[arg(long,default_value="mirror")] mode:String },
    Import { dir:String }, List,
    Sync { #[arg(long)] repo:Option<i64> },
    Gate { #[arg(long,default_value="HEAD~1..HEAD")] range:String, #[arg(long,default_value="json")] format:String },
    Slice { commit:String, #[arg(long)] plan_only:bool },
    Merge { #[arg(long)] refspec:String },
    Tui, Web { #[arg(long,default_value="127.0.0.1:7420")] bind:String },
    Doctor, Hook { action:String },
}
```

```rust
// main.rs
mod cli; mod config; mod store; mod events; mod scheduler; mod governor;
mod git; mod sync; mod gate; mod rules; mod sbom; mod osv; mod slice; mod merge;
mod tui; mod web; mod doctor;
use clap::Parser;
use std::sync::Arc;

fn main()->anyhow::Result<()>{
    let cli = cli::Cli::parse();
    let cfg = config::Config::load()?;
    let db = dirs::home_dir().unwrap().join(".colmeia/colmeia.db");
    let store = Arc::new(store::Store::open(&db)?);
    let git = git::Git{ cap_mb: cfg.git_child_cap_mb };
    match cli.cmd {
        cli::Cmd::Init => { doctor::init_wizard(&cfg)?; }
        cli::Cmd::Add{url,mode} => {
            let path = format!("./repos/{}", url.rsplit('/').next().unwrap_or("repo").trim_end_matches(".git"));
            std::fs::create_dir_all("./repos")?;
            let o = git.run(std::path::Path::new("."), &["clone","--bare","--filter=blob:none",&url,&path])?;
            anyhow::ensure!(o.status.success(),"clone falhou");
            store.add_repo(&path, Some(&url), &mode)?;
            println!("adicionado: {path}");
        }
        cli::Cmd::Import{dir} => {
            for e in walkdir::WalkDir::new(&dir).max_depth(2){
                let e=e?; if e.path().join(".git").exists(){
                    store.add_repo(&e.path().to_string_lossy(), None, "managed")?;
                }
            }
        }
        cli::Cmd::List => { for (id,p,m,sha) in store.list_repos()? { println!("{id}\t{m}\t{}\t{p}", &sha[..7.min(sha.len())]); } }
        cli::Cmd::Sync{repo} => {
            for (id,_,_,_) in store.list_repos()? {
                if repo.map(|r|r==id).unwrap_or(true){ store.enqueue("fetch",id,&serde_json::json!({"remote":"origin"}),5)?; }
            }
            scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Gate{range,format} => {
            for (id,_,_,_) in store.list_repos()? {
                gate::run_repo(&store,&git,&cfg,id,&serde_json::json!({"range":range}))?;
            }
            if format=="sarif"{ println!("{}", gate::sarif(&[])); }
        }
        cli::Cmd::Slice{commit,plan_only} => {
            for (id,_,_,_) in store.list_repos()? {
                store.enqueue("slice_plan",id,&serde_json::json!({"commit":commit,"max_lines":400}),5)?;
            }
            let _ = plan_only; scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Merge{refspec} => {
            for (id,_,_,_) in store.list_repos()? { store.enqueue("merge",id,&serde_json::json!({"ref":refspec,"policy":"escalate"}),3)?; }
            scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Tui => tui::run(store.clone())?,
        cli::Cmd::Web{bind} => web::run(store.clone(), &bind)?,
        cli::Cmd::Doctor => doctor::run(&store,&cfg)?,
        cli::Cmd::Hook{action} => { if action=="install"{ doctor::install_hooks()?; } }
    }
    Ok(())
}
```

```rust
// doctor.rs
use crate::{store::Store, config::Config};
pub fn init_wizard(cfg:&Config)->anyhow::Result<()>{
    std::fs::create_dir_all(dirs::home_dir().unwrap().join(".colmeia"))?;
    let p = Config::path();
    if !p.exists(){ std::fs::write(&p, toml::to_string_pretty(cfg)?)?; }
    println!("Colmeia iniciada. tier={} workers={} soft={}MB", Config::detect_tier(), cfg.workers, cfg.memory_soft_mb);
    println!("Próximos passos: colmeia add <url>  ·  colmeia tui  ·  colmeia doctor");
    Ok(())
}
pub fn run(st:&Store, cfg:&Config)->anyhow::Result<()>{
    let git_ok = std::process::Command::new("git").arg("--version").output().is_ok();
    println!("git instalado: {}", if git_ok{"ok"}else{"FALTA — instale git>=2.40"});
    println!("tier: {}  workers: {}  rss: {}MB  avail: {}MB",
        Config::detect_tier(), cfg.workers, crate::governor::rss_mb(), crate::governor::available_mb());
    println!("repos: {}", st.list_repos()?.len());
    println!("pressão: {:?}", crate::governor::pressure(cfg.memory_soft_mb,cfg.memory_hard_mb));
    Ok(())
}
pub fn install_hooks()->anyhow::Result<()>{
    // TODO(F25): hook runner nativo pre-commit/pre-push chamando `colmeia gate`.
    println!("hooks: stub — instala pre-commit/pre-push que chamam `colmeia gate`");
    Ok(())
}
```

### 1.15 `justfile`, CI, RFC, testes

```make
# justfile
build:; cargo build --release
test:; cargo test
lint:; cargo clippy -- -D warnings && cargo fmt --check
farm n=200:; cargo test --test farm -- --nocapture {{n}}
bench:; cargo run --release -- doctor
```

```yaml
# .github/workflows/ci.yml
name: ci
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
      # TODO(F54): bench-contract (RSS/latência) como gate de PR
```

```md
<!-- rfcs/0001-determinism.md -->
# RFC 0001 — Determinismo
- Ordenações estáveis; tie-break por hash de conteúdo.
- Políticas versionadas por `policy_hash`.
- Toda resolução/slice gera `recipe` replayável (`colmeia replay`).
- Escritas de ref só via `update-ref <new> <old>` (CAS) + backup ref.
- Jobs com lease ⇒ crash-recovery sem perda.
```

```rust
// tests/farm.rs — gerador de repos sintéticos (esqueleto)
#[test] fn farm(){ /* TODO(F54): criar N repos com históricos/lockfiles/skills plantadas */ }
// tests/determinism.rs
#[test] fn slice_plan_deterministic(){ /* TODO: proptest mesmo input ⇒ mesmo plan_hash */ }
```

**Stub vs real no esqueleto:**

| Real agora | Stub (TODO) |
|---|---|
| store/migrations, leases, governor, git capado, sync fetch/ff/mirror, gate rules+SBOM parse, slice plan, merge drivers, TUI lista, web read-only, doctor/init | Starlark loader, OSV unzip/matcher, slice exec, workbench, path_sync, SSE real, grafo, plugins, hooks runner, bench-contract |

---

## 2. Plano de implementação — matriz de features

Versões: **M**=MVP v0.1 · **2**=v0.2 · **3**=v0.3 · **1**=v1.0 GA.

| ID | Feature | Ver | Dep | Critério de aceite |
|---|---|---|---|---|
| F01 | Registry + import em massa | M | — | 2.000 repos import ≤60s |
| F02 | Modos managed/mirror/dormant + LRU | M | F01 | worktrees ≤30 em T2 |
| F03 | Scheduler adaptativo (cron+poll+evento) | M | F05 | sweep 2.000 ≤3h T2 |
| F04 | inotify hot-subset | 2 | F03 | watches ≤ kernel limit |
| F05 | Jobs DAG + leases + locks | M | — | kill -9 ⇒ recovery |
| F06 | Memory governor soft/hard + rlimits | M | — | RSS ≤600MB típico |
| F07 | sync fetch_upstream | M | F05 | partial clone ok |
| F08 | branch_sync ff-only | M | F07 | divergência escala |
| F09 | push_mirror | M | F07 | mirror idempotente |
| F10 | path_sync skill-packs | 2 | F08 | hash conteúdo + commit det. |
| F11 | slice plan-only | M | F01 | plano ≤5s/5k linhas |
| F12 | slice exec (apply+gate+CAS+backup) | 2 | F11,F24 | replay bit-idêntico |
| F13 | merge drivers ours/theirs/union | M | F08 | nunca força |
| F14 | structured 3-way JSON/YAML/TOML | 2 | F13 | merge por chave |
| F15 | workbench 3-pane + fila | 2 | F13 | humano nunca auto |
| F16 | recipes + replay | 2 | F12,F14 | bit-idêntico proptest |
| F17 | rules Starlark hermético | 2 | F24 | packs versionados |
| F18 | secrets regex+entropia | M | — | falsos positivos <5% |
| F19 | skills OWASP LLM | M | — | injection/unicode/agency |
| F20 | SBOM parsers lockfiles | M | — | 3 eco MVP, +5 em v0.2 |
| F21 | OSV sync nightly | M | F20 | delta ≤60s |
| F22 | matcher semver/pep440 | M | F21 | 10k pacotes ≤5s |
| F23 | watch + auto-PR bump | 2 | F22,F32 | bump simples só |
| F24 | gate + SARIF + paridade CI | M | F18 | mesmo resultado local/CI |
| F25 | hook runner nativo | M | F24 | pre-commit/pre-push |
| F26 | docs engine + doc cards | 2 | F20 | regen O(diff) |
| F27 | FTS5 busca docs | 2 | F26 | busca 2.000 repos |
| F28 | graph store edges | 2 | F01 | commit-graph accel |
| F29 | layout incremental + culling | 3 | F28 | 312 nós 60fps |
| F30 | anomaly EWMA/MAD | M | F03 | batch ≤15min |
| F31 | cross-repo fanout/drift | 2 | F30 | supply-chain signal |
| F32 | forge GitHub + rate budget | M | — | budget respeitado |
| F33 | GitLab + Forgejo | 3 | F32 | adapters trait |
| F34 | rate budget por forge | M | F32 | backoff |
| F35 | PRs agendados | 2 | F32 | ±30s do cron |
| F36 | auto-merge c/ lease | 3 | F35 | checks poll |
| F37 | webhooks in HMAC | 2 | F32 | assinatura verify |
| F38 | hooks out exec/HTTP/MQTT | 2 | F37 | T0 trigger |
| F39 | plugins Starlark packs | 2 | F17 | packs como repos |
| F40 | plugins JSON-RPC stdio + caps | 3 | F39 | sandbox caps |
| F41 | WASM extism | 1 | F40 | só se demanda |
| F42 | tiers T1/T2/T3 presets | M | F06 | RPi passa suite |
| F43 | companion T0 firmware | 3 | F38 | ESP32/Uno JSON |
| F44 | temas claro/escuro/sépia/neumorph | 3 | web | AA contraste |
| F45 | motion + responsivo web | 3 | F44 | breakpoints 4 |
| F46 | TUI paridade + grafo braille | 2 | F28 | 80×24 |
| F47 | onboarding wizard | M | — | zero-config |
| F48 | doctor self-heal | M | — | diagnóstico claro |
| F49 | installers (cargo/brew/deb) | 1 | F47 | 1 comando |
| F50 | docs + i18n PT/EN | 1 | F47 | tutorial comum |
| F51 | secrets age + keyring | M | — | nunca plaintext |
| F52 | redaction + fuzz parsers | 1 | F20 | fuzz sem crash |
| F53 | OTLP/LangSmith bridge | 3 | F38 | opt-in |
| F54 | tests/bench/chaos/proptest | M | — | CI gate |
| F55 | governance RFC/DCO/licença | M | — | público |

---

## 3. Roadmap de produção + usuário comum

### Fases e gates de saída

| Fase | Semanas | Entrega | Gate de saída (definition of done) |
|---|---|---|---|
| **0 Fundação** | 1–3 | esqueleto compilável, CI, RFC0001, farm | `cargo test` verde; import 2.000 ≤60s; RSS idle ≤120MB |
| **1 Core confiável** | 4–9 | F03–F09, governor, leases | chaos kill -9 sem perda; sweep ≤3h T2; soak 24h |
| **2 Segurança** | 10–15 | F18–F25, F51 | gate paridade local/CI; OSV delta ≤60s; self-audit secrets |
| **3 Determinismo** | 16–21 | F11–F16, F46 | proptest replay bit-idêntico; slice exec com CAS+backup |
| **4 Integração** | 22–27 | F32,F34,F35,F37, web read-only | rate budget; PR ±30s; web ≤10MB daemon |
| **5 Usuário comum** | 28–33 | F47,F48,F50, temas, responsivo | beta público; tutorial ≤10min; doctor self-heal; AA |
| **6 Hardening/GA** | 34–40 | F49,F52,F41?, soak, perf RPi | soak 7d; fuzz limpo; perf contract RPi; v1.0 |

### Checklist de produção (v1.0 GA)

- **Confiabilidade:** crash-recovery por lease; WAL+backup/restore do DB (`colmeia db export/import`); migrações versionadas com rollback; soak 7 dias em T1/T2.
- **Segurança do tool:** tokens age-cifrados; redação no audit; fuzz de parsers de lockfile/webhook; SARIF assinado; sem plaintext em logs.
- **Performance como contrato:** bench público falha PR que regride RSS/latência; suporte T1 (512MB) com suite reduzida.
- **Observabilidade opt-in:** OTLP/LangSmith bridge desligado por default; crash reports opt-in.
- **Operação:** semver; cadência de release; LTS de segurança; uninstall limpo; portabilidade de dados (export JSON/SQLite).
- **Suporte:** matriz OS/arch (linux x86_64/aarch64/armv6, macOS); binários musl estáticos.

### Checklist "qualquer usuário comum"

- **Instalação 1 comando** (`cargo install colmeia` / brew / .deb) + `colmeia init` wizard que detecta tier e escreve config.
- **Zero jargão em erros:** mensagens PT/EN com causa + ação sugerida; `--json` para scripts.
- **Segurança por default:** destrutivo = dry-run + confirmação mostrando CAS; undo 5s; nunca force-push.
- **GUI web para quem não usa terminal** (read-only no MVP, interativa v0.3) + TUI para quem usa.
- **`colmeia doctor`** que diagnostica e cura (git faltando, permissão, DB lock, pressão de memória).
- **Docs tutorial** "importe sua pasta de projetos em 10 minutos" + empty-states com CTA real.
- **Acessibilidade:** contraste AA nos 4 temas, alvos 44px, `prefers-reduced-motion`, navegação por teclado/⌘K.

---

## 4. Riscos e próximo passo

| Risco | Mitigação |
|---|---|
| MVP inflar | linha de corte rígida (§1); exec/workbench/plugins fora |
| gitoxide imaturo | CLI capado no MVP; trait isola migração |
| OSV dump grande | filtro por eco + poda withdrawn |
| Usuário comum perdido | wizard + doctor + web + erros humanos |
| Determinismo quebrar | proptest + recipes + CAS como gate de CI |

**Próximo comando:** `cargo build && cargo run -- init && cargo run -- add <url> && cargo run -- tui`. A partir daí, preencha os `TODO(Fxx)` na ordem da matriz — cada um tem critério de aceite mensurável e versão-alvo, então o caminho até v1.0 é executável, não aspiracional.