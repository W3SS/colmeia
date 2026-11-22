PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-32768;
PRAGMA temp_store=FILE;

CREATE TABLE IF NOT EXISTS repos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT UNIQUE NOT NULL,
  url TEXT,
  mode TEXT CHECK(mode IN ('managed','mirror','dormant')) DEFAULT 'mirror',
  forge TEXT,
  default_branch TEXT,
  head_sha TEXT,
  priority INTEGER DEFAULT 5,
  last_activity INTEGER,
  state TEXT DEFAULT 'idle'
);

CREATE TABLE IF NOT EXISTS scan_roots (
  path TEXT PRIMARY KEY,
  last_scan INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_policies (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  src_repo INTEGER,
  dst_repo INTEGER,
  kind TEXT CHECK(kind IN ('fetch_upstream','push_mirror','branch_sync','path_sync')),
  refspec TEXT,
  path_glob TEXT,
  cron TEXT,
  on_event TEXT,
  conflict_policy TEXT DEFAULT 'escalate',
  enabled INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS jobs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL,
  repo_id INTEGER,
  payload_json TEXT NOT NULL,
  status TEXT DEFAULT 'queued',
  priority INTEGER DEFAULT 5,
  lease_until INTEGER,
  attempts INTEGER DEFAULT 0,
  depends_on INTEGER,
  created_at INTEGER,
  finished_at INTEGER,
  error TEXT
);
CREATE INDEX IF NOT EXISTS jobs_dispatch ON jobs(status, priority, created_at);

CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER,
  repo_id INTEGER,
  kind TEXT,
  payload_json TEXT
);

CREATE TABLE IF NOT EXISTS findings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id INTEGER,
  blob_sha TEXT,
  rule_id TEXT,
  severity TEXT,
  path TEXT,
  line INTEGER,
  message TEXT,
  owasp TEXT,
  status TEXT DEFAULT 'open',
  first_seen INTEGER,
  last_seen INTEGER,
  UNIQUE(repo_id, blob_sha, rule_id, path, line)
);

CREATE TABLE IF NOT EXISTS sbom (
  repo_id INTEGER,
  commit_sha TEXT,
  ecosystem TEXT,
  name TEXT,
  version TEXT,
  purl TEXT,
  direct INTEGER,
  PRIMARY KEY(repo_id, commit_sha, ecosystem, name, version)
);

CREATE TABLE IF NOT EXISTS vulns (
  id TEXT PRIMARY KEY,
  aliases_json TEXT,
  ecosystem TEXT,
  name TEXT,
  ranges_json TEXT,
  severity TEXT,
  fixed_json TEXT,
  modified INTEGER,
  withdrawn INTEGER
);

CREATE TABLE IF NOT EXISTS baselines (
  repo_id INTEGER,
  metric TEXT,
  ewma REAL,
  mad REAL,
  n INTEGER,
  updated_at INTEGER,
  PRIMARY KEY(repo_id, metric)
);

CREATE TABLE IF NOT EXISTS recipes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id INTEGER,
  kind TEXT,
  input_hashes TEXT,
  policy_hash TEXT,
  output_hash TEXT,
  plan_json TEXT,
  created_at INTEGER
);

CREATE TABLE IF NOT EXISTS audit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER,
  actor TEXT,
  action TEXT,
  target TEXT,
  details_json TEXT
);

-- F02: cache de worktrees com despejo LRU
CREATE TABLE IF NOT EXISTS worktrees (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id INTEGER NOT NULL,
  branch TEXT NOT NULL,
  path TEXT UNIQUE NOT NULL,
  purpose TEXT DEFAULT 'slice',
  last_used INTEGER,
  created_at INTEGER
);
CREATE INDEX IF NOT EXISTS worktrees_lru ON worktrees(last_used);

-- F26/F27: doc cards + índice FTS5
CREATE TABLE IF NOT EXISTS docs (
  repo_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  title TEXT,
  content TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  updated_at INTEGER,
  PRIMARY KEY(repo_id, path)
);
CREATE VIRTUAL TABLE IF NOT EXISTS doc_fts USING fts5(repo_path UNINDEXED, title, content);

-- F51: cofre de segredos (valores sempre cifrados, nunca plaintext)
CREATE TABLE IF NOT EXISTS secrets (
  name TEXT PRIMARY KEY,
  ciphertext TEXT NOT NULL,
  nonce TEXT NOT NULL,
  updated_at INTEGER
);

-- F32/F34: orçamento de requisições por forge
CREATE TABLE IF NOT EXISTS rate_budget (
  forge TEXT NOT NULL,
  window_start INTEGER NOT NULL,
  used INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(forge, window_start)
);

-- F28: arestas do grafo de repositórios
CREATE TABLE IF NOT EXISTS graph_edges (
  src_repo INTEGER NOT NULL,
  dst_repo INTEGER NOT NULL,
  weight REAL NOT NULL,
  kind TEXT DEFAULT 'cochange',
  updated_at INTEGER,
  PRIMARY KEY(src_repo, dst_repo, kind)
);

-- F38: entregas de hooks externos
CREATE TABLE IF NOT EXISTS hook_deliveries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hook TEXT NOT NULL,
  event TEXT NOT NULL,
  status TEXT NOT NULL,
  detail TEXT,
  ts INTEGER
);

-- gh-ci: dedup determinístico de runs de CI já relatados
CREATE TABLE IF NOT EXISTS ci_runs (
  run_id INTEGER PRIMARY KEY,
  repo TEXT NOT NULL,
  workflow TEXT,
  conclusion TEXT,
  seen_at INTEGER NOT NULL
);
