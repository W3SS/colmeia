#!/usr/bin/env node
/* Teste de integração da API read-only de exploração (zero dependências).
   Sobe o dev-server numa porta alta, valida /api/tree e /api/repo contra o
   diretório real (somente leitura — nenhuma rota muta o filesystem). */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join, sep } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const PORT = 7199;
const BASE = `http://127.0.0.1:${PORT}`;
let failures = 0;
const check = (name, cond) => {
  console.log((cond ? "PASS " : "FAIL ") + name);
  if (!cond) failures++;
};

const srv = spawn(process.execPath, [join(here, "dev-server.mjs"), "--port", String(PORT)], {
  stdio: ["ignore", "pipe", "pipe"],
});
await new Promise((resolve, reject) => {
  const t = setTimeout(() => reject(new Error("servidor não subiu")), 15000);
  srv.stdout.on("data", (d) => { if (String(d).includes("http://")) { clearTimeout(t); resolve(); } });
  srv.stderr.on("data", (d) => process.stderr.write(d));
});

const get = async (path) => {
  const r = await fetch(BASE + path, { cache: "no-store" });
  let body = null;
  try { body = await r.json(); } catch { /* corpo não-JSON */ }
  return { status: r.status, body };
};

try {
  // 1. Raiz: lista pastas e repos reais
  const root = await get("/api/tree?path=");
  check("raiz ok", root.status === 200 && root.body?.ok === true);
  check("raiz é um diretório de repos", typeof root.body?.root === "string" && root.body.root.endsWith(sep + "repos"));
  check(">= 30 repos no nível raiz", (root.body?.repos?.length ?? 0) >= 30);
  check("pasta 'projects' listada", root.body?.dirs?.includes("projects"));
  check("contagem de arquivos soltos", typeof root.body?.files === "number");

  // 2. Subnível: projects/ misto — repos git e pastas comuns
  const sub = await get("/api/tree?path=projects");
  check("projects ok", sub.status === 200 && sub.body?.ok === true);
  check(">= 10 repos dentro de projects/", (sub.body?.repos?.length ?? 0) >= 10);
  // após a migração do projeto para o repo git `colmeia`, colmeia-app pode
  // ser symlink (aparece como repo) ou pasta, conforme o checkout
  check("colmeia-app presente em projects/",
    sub.body?.dirs?.includes("colmeia-app") || sub.body?.repos?.includes("colmeia-app"));

  // 3. Detalhe de um repo real
  const name = root.body.repos[0];
  const repo = await get("/api/repo?path=" + encodeURIComponent(name));
  check("repo ok", repo.status === 200 && repo.body?.ok === true);
  check("HEAD hex", /^[0-9a-f]{40}$/.test(repo.body?.head || ""));
  check("branch atual não vazia", (repo.body?.branch || "").length > 0);
  check("branches locais listadas", (repo.body?.branches?.length ?? 0) >= 1);
  check("último commit tem assunto (>5 chars, sem split por caractere)", (repo.body?.subject || "").length > 5);
  check("dirty é número", typeof repo.body?.dirty === "number");

  // 4. Grafo gitflow: commits com pais e refs
  const log = await get("/api/log?path=" + encodeURIComponent(name) + "&n=40");
  check("gitflow log ok", log.status === 200 && log.body?.ok === true);
  check("commits têm sha e subject", (log.body?.commits?.length ?? 0) >= 1 &&
    /^[0-9a-f]{40}$/.test(log.body.commits[0].sha || "") && (log.body.commits[0].subject || "").length > 0);
  check("parents é array por commit", log.body.commits.every((c) => Array.isArray(c.parents)));
  check("limite n respeitado", log.body.commits.length <= 40);
  check("não-repo no /api/log rejeitado (400)",
    (await get("/api/log?path=" + encodeURIComponent("projects"))).status === 400);

  // 5. Scan recursivo de toda a frota
  const scan = await get("/api/scan");
  check("scan ok", scan.status === 200 && scan.body?.ok === true);
  check(">= 45 repos descobertos no total", (scan.body?.repos?.length ?? 0) >= 45);
  check("cada repo tem path e cluster", (scan.body?.repos ?? []).every((r) => r.path && r.cluster));
  check("projects/lia presente no scan", scan.body?.repos?.some((r) => r.path === "projects/lia"));

  // 6. Segurança: traversal, fora do root, não-repo, método mutante
  const trav = await get("/api/tree?path=" + encodeURIComponent("../../.."));
  check("traversal bloqueado (403)", trav.status === 403);
  const notRepo = await get("/api/repo?path=" + encodeURIComponent("projects"));
  check("não-repo rejeitado (400)", notRepo.status === 400 && notRepo.body?.ok === false);
  const missing = await get("/api/tree?path=nao-existe-12345");
  check("caminho inexistente (404)", missing.status === 404);
  const post = await fetch(BASE + "/api/tree?path=", { method: "POST" });
  check("POST rejeitado (405, read-only)", post.status === 405);
} finally {
  srv.kill();
}

if (failures) { console.error(`\n${failures} falha(s)`); process.exit(1); }
console.log("\nTestes da API de exploração: 100% OK");
