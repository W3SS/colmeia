#!/usr/bin/env node
/* Servidor estático + API read-only de exploração de diretório (só Node built-ins).
   Uso: node scripts/dev-server.mjs [--root web] [--host 127.0.0.1] [--port 7100]
   Env: COLMEIA_REPOS_ROOT (default: ~/Ubuntu/home/Documents/repos se existir,
        senão ~/Documents/repos)

   Contrato da API (SOMENTE leitura — espelha o contrato dos testes Rust):
     GET /api/tree?path=<rel>  → pastas e repos git (1 nível, lazy)
     GET /api/repo?path=<rel>  → HEAD, branch, branches, último commit, remote, dirty
     GET /api/log?path=<rel>&n=N → commits topo-order com pais e refs (grafo gitflow)
   Nenhuma rota muta o filesystem; `git` é chamado só com subcomandos read-only. */
import { createServer } from "node:http";
import { existsSync } from "node:fs";
import { readFile, readdir, stat } from "node:fs/promises";
import { join, extname, resolve, sep } from "node:path";
import { homedir } from "node:os";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const run = promisify(execFile);
const args = process.argv.slice(2);
const opt = (name, dflt) => {
  const i = args.indexOf("--" + name);
  return i >= 0 && args[i + 1] ? args[i + 1] : dflt;
};
const root = resolve(opt("root", "web"));
const host = opt("host", "127.0.0.1");
const port = Number(opt("port", "7100"));
// Sem caminho hardcoded: env > candidatos relativos ao $HOME (sem expor usuário).
const home = homedir();
const REPOS_ROOT = resolve(
  process.env.COLMEIA_REPOS_ROOT ||
    [
      join(home, "Ubuntu", "home", "Documents", "repos"),
      join(home, "Documents", "repos"),
    ].find((p) => existsSync(p)) ||
    join(home, "Documents", "repos")
);
const SKIP = new Set(["node_modules", "target", ".git", "__pycache__", ".next", "dist", "build"]);

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".json": "application/json",
  ".png": "image/png",
  ".woff2": "font/woff2",
};

const json = (res, code, body) => {
  res.writeHead(code, { "content-type": "application/json; charset=utf-8", "cache-control": "no-store" });
  res.end(JSON.stringify(body));
};

/* Resolve `rel` contra REPOS_ROOT e exige que fique dentro dele (anti-traversal). */
function safeJoin(rel) {
  const p = resolve(REPOS_ROOT, "." + sep + String(rel || ""));
  if (p !== REPOS_ROOT && !p.startsWith(REPOS_ROOT + sep)) return null;
  return p;
}

async function isGitRepo(dir) {
  try {
    const s = await stat(join(dir, ".git"));
    return s.isDirectory() || s.isFile(); // .git arquivo = worktree/submodule
  } catch { return false; }
}

/* Lista 1 nível: pastas, repos git e contagem de arquivos. */
async function listLevel(absDir) {
  const entries = await readdir(absDir, { withFileTypes: true });
  const dirs = [], repos = [];
  let files = 0;
  for (const e of entries) {
    if (SKIP.has(e.name) || e.name.startsWith(".")) continue;
    if (!e.isDirectory()) { files++; continue; }
    if (await isGitRepo(join(absDir, e.name))) repos.push(e.name);
    else dirs.push(e.name);
  }
  return { dirs: dirs.sort((a, b) => a.localeCompare(b)), repos: repos.sort((a, b) => a.localeCompare(b)), files };
}

/* git read-only com timeout generoso (o mount do diretório pode ser lento);
   qualquer falha vira string vazia / 0. GIT_OPTIONAL_LOCKS=0 evita locks. */
async function gitRO(absDir, gargs, timeout = 25000) {
  try {
    const { stdout } = await run("git", gargs, { cwd: absDir, timeout, maxBuffer: 1 << 20,
      env: { ...process.env, GIT_OPTIONAL_LOCKS: "0" } });
    return stdout.trim();
  } catch { return ""; }
}

async function repoInfo(absDir, rel) {
  const [head, branch, branchOut, logOut, remoteOut, statusOut] = await Promise.all([
    gitRO(absDir, ["rev-parse", "HEAD"]),
    gitRO(absDir, ["symbolic-ref", "--short", "HEAD"]),
    gitRO(absDir, ["branch", "--list", "--format=%(refname:short)"]),
    gitRO(absDir, ["log", "-1", "--format=%s%x00%an%x00%ct"]),
    gitRO(absDir, ["remote", "get-url", "origin"]),
    gitRO(absDir, ["status", "--porcelain"]),
  ]);
  const [subject, author, ts] = (logOut || "").split(String.fromCharCode(0));
  const branches = branchOut ? branchOut.split("\n").filter(Boolean).slice(0, 40) : [];
  const dirty = statusOut ? statusOut.split("\n").filter(Boolean).length : 0;
  return {
    ok: true, root: REPOS_ROOT, path: rel, name: rel.split("/").pop(),
    head, branch, branches, subject, author,
    ts: ts ? Number(ts) : null, remote: remoteOut, dirty,
  };
}

/* Varredura recursiva SÓ de filesystem (sem git → rápida mesmo em mount lento).
   BFS com limite de profundidade 3 e de quantidade; não desce dentro de repo. */
async function scanAll() {
  const repos = [];
  const pending = [[REPOS_ROOT, 0, ""]];
  while (pending.length && repos.length < 400) {
    const [dir, depth, relBase] = pending.shift();
    let entries;
    try { entries = await readdir(dir, { withFileTypes: true }); } catch { continue; }
    for (const e of entries) {
      if (repos.length >= 400) break;
      if (SKIP.has(e.name) || e.name.startsWith(".")) continue;
      if (!e.isDirectory()) continue;
      const rel = relBase ? relBase + "/" + e.name : e.name;
      if (await isGitRepo(join(dir, e.name))) {
        repos.push({ path: rel, name: e.name, cluster: relBase ? relBase.split("/")[0] : "(raiz)" });
        continue;
      }
      if (depth < 3) pending.push([join(dir, e.name), depth + 1, rel]);
    }
  }
  repos.sort((a, b) => a.path.localeCompare(b.path));
  return { ok: true, root: REPOS_ROOT, repos };
}

async function handleApi(req, res, url) {
  if (req.method !== "GET") return json(res, 405, { ok: false, error: "método não permitido (API é read-only)" }), true;
  const rel = url.searchParams.get("path") || "";
  const abs = safeJoin(rel);
  if (!abs) return json(res, 403, { ok: false, error: "caminho fora do diretório permitido" }), true;
  let st;
  try { st = await stat(abs); } catch { return json(res, 404, { ok: false, error: "caminho não encontrado" }), true; }
  if (!st.isDirectory()) return json(res, 400, { ok: false, error: "não é um diretório" }), true;

  if (url.pathname === "/api/scan") {
    return json(res, 200, await scanAll()), true;
  }
  if (url.pathname === "/api/tree") {
    const level = await listLevel(abs);
    return json(res, 200, { ok: true, root: REPOS_ROOT, path: rel, ...level }), true;
  }
  if (url.pathname === "/api/repo") {
    if (!(await isGitRepo(abs))) return json(res, 400, { ok: false, error: "não é um repositório git" }), true;
    return json(res, 200, await repoInfo(abs, rel)), true;
  }
  /* Grafo gitflow estilo GitKraken: commits em ordem topológica com pais e refs. */
  if (url.pathname === "/api/log") {
    if (!(await isGitRepo(abs))) return json(res, 400, { ok: false, error: "não é um repositório git" }), true;
    const n = Math.min(400, Math.max(10, parseInt(url.searchParams.get("n") || "120", 10) || 120));
    const fmt = "%H%x00%P%x00%s%x00%an%x00%ct%x00%D";
    const out = await gitRO(abs, ["log", "--topo-order", "--max-count=" + n, "--format=" + fmt]);
    const commits = out
      ? out.split("\n").filter(Boolean).map((line) => {
          const [sha, parents, subject, author, ts, refs] = line.split(String.fromCharCode(0));
          return { sha, parents: parents ? parents.split(" ") : [], subject, author,
            ts: Number(ts) || 0, refs: refs || "" };
        })
      : [];
    const brOut = await gitRO(abs, ["branch", "--list", "--format=%(refname:short)"]);
    return json(res, 200, { ok: true, path: rel, count: commits.length,
      commits, branches: brOut ? brOut.split("\n").filter(Boolean).slice(0, 40) : [] }), true;
  }
  return false; // rota /api desconhecida
}

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url, "http://x");
    if (url.pathname.startsWith("/api/")) {
      if (await handleApi(req, res, url)) return;
      return json(res, 404, { ok: false, error: "rota de API desconhecida" });
    }
    let path = decodeURIComponent(url.pathname);
    if (path === "/") path = "/index.html";
    const file = join(root, path);
    if (!resolve(file).startsWith(root)) throw new Error("fora do root");
    const body = await readFile(file);
    res.writeHead(200, {
      "content-type": MIME[extname(file)] || "application/octet-stream",
      "cache-control": "no-store", // dev server: nunca servir cópia em cache
    });
    res.end(body);
  } catch {
    res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    res.end("404");
  }
});

server.listen(port, host, () => {
  console.log(`Colmeia Web GUI → http://${host}:${port}/ (root: ${root})`);
  console.log(`API read-only de repos → ${REPOS_ROOT}`);
});
