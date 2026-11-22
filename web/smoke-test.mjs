#!/usr/bin/env node
/* Smoke test determinístico da Web GUI da Colmeia (zero dependências).
   Valida estrutura, telas, temas e a sintaxe do script embutido. */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(here, "index.html"), "utf8");
let failures = 0;
const check = (name, cond) => {
  console.log((cond ? "PASS " : "FAIL ") + name);
  if (!cond) failures++;
};

// 1. Documento base íntegro
check("doctype html", html.startsWith("<!doctype html>"));
check("fecha </html>", html.trimEnd().endsWith("</html>"));
check("lang pt-BR", /<html lang="pt-BR"/.test(html));
check("viewport responsivo", /name="viewport"/.test(html));

// 2. Telas obrigatórias
for (const [fn, nome] of [
  ["screenOverview", "Visão Geral"], ["screenRepos", "Repositórios"], ["screenExplorar", "Explorar"], ["screenSync", "Sync"],
  ["screenSlice", "Slice"], ["screenMerge", "Merge"], ["screenSecurity", "Segurança"],
  ["screenDocs", "Docs"], ["screenGraph", "Grafo"], ["screenBranches", "Branches"],
  ["screenPRs", "PRs"], ["screenAnomaly", "Anomalias"], ["screenTests", "Testes"],
]) check(`tela ${nome}`, html.includes(`function ${fn}`));

// 3. Branching: dados e renderizador
check("grafo de commits de branches", html.includes("function buildBranchGraph"));
check("4 repositórios de exemplo", (html.match(/hotfix:(?:true|false)/g) || []).length === 4);
check("convenções feature/bugfix/hotfix", ["feature/*", "bugfix/*", "hotfix/*"].every((s) => html.includes(s)));

// 4. Automação de testes: 24 checkpoints
const tests24 = html.match(/const TESTS24=\[([\s\S]*?)\];/);
check("checklist de 24 pontos", tests24 && (tests24[1].match(/,\s*\n \[/g) || []).length + 1 === 24);

// 5. Temas: ≥15 cyberpunk + 4 base
const cyber = (html.match(/g:"Cyberpunk"/g) || []).length;
check(`temas cyberpunk >= 15 (achou ${cyber})`, cyber >= 15);
for (const t of ["escuro", "claro", "sepia", "neumorph"]) check(`tema base ${t}`, html.includes(`${t}:{n:`));

// 6. Hambúrguer responsivo
check("botão hambúrguer", html.includes('id="burger"'));
check("abre/fecha menu", html.includes("function toggleNav") && html.includes("function closeNav"));
check("trilho escuro sempre visível + painel que expande", html.includes('class="rail"') && html.includes('class="npanel"') && html.includes('nav.open .npanel{transform:none;pointer-events:auto}'));
check("painel escondido não intercepta cliques no trilho", html.includes('nav .npanel{') && html.includes('pointer-events:none'));
check("nav e estúdio fora do .app (scrim não cobre cliques)", !/class="app"[^>]*>\s*<nav/.test(html));
check("Estúdio fechado por padrão no canto superior direito", html.includes('body.studio-open .studio{transform:none}') && html.includes('top:56px;right:0'));
check("toggle do Estúdio", html.includes("function toggleStudio") && html.includes("function closeStudio") && html.includes('data-act="toggle-studio"'));
check("botão Estúdio é o último do header", html.indexOf('data-act="add-repo"') < html.indexOf('class="btn sm studio-btn"'));
// 6b. Gitflow estilo GitKraken
check("aba GitFlow no drawer", html.includes('["gitflow","GitFlow"]') && html.includes('id="gfWrap"'));
check("API /api/log usada pelo gitflow", html.includes('/api/log?path='));
check("layout de lanes + desenho do grafo", html.includes("function gfLayout") && html.includes("function drawGitflow"));
check("rótulos booleanos do Estúdio", html.includes('if(typeof v==="boolean")return v?"on":"off"'));

// 6. Explorar: árvore do diretório real via API read-only
check("tela Explorar no SCREENS", html.includes("explorar:screenExplorar"));
check("ações fs-toggle/fs-open/fs-import", ["fs-toggle", "fs-open", "fs-import", "fs-import-visible"].every((a) => html.includes(`data-act="${a}"`) || html.includes(`case "${a}"`)));
check("lazy load /api/tree", html.includes("/api/tree?path="));
check("detalhe /api/repo", html.includes("/api/repo?path="));
check("ícones caret/folder", html.includes("caret:'") && html.includes("folder:'"));

// 6b. Frota real: todas as telas refletem o diretório
check("motor de frota real (realScan/applyRealFleet)", html.includes("function realScan") && html.includes("function applyRealFleet"));
check("grafo reconstruível a partir dos repos reais", html.includes("function rebuildFleetGraph"));
check("cache da frota no localStorage", html.includes("colmeia-repos-real"));
check("scan recursivo /api/scan", html.includes("/api/scan"));
check("branches ligados aos repos reais", html.includes("function syncBranchRepos"));

// 7. Script embutido compila
const m = html.match(/<script>([\s\S]*?)<\/script>/);
check("um único bloco <script>", !!m);
if (m) {
  try { new Function(m[1]); check("sintaxe JS válida", true); }
  catch (e) { check(`sintaxe JS válida (${e.message})`, false); }
}

// 8. Leveza: sem frameworks/CDN obrigatório de runtime
check("sem <script src=", !/<script[^>]+src=/.test(html));
check("arquivo único < 220 KB", Buffer.byteLength(html) < 220 * 1024);

// 9. Balanceamento de tags estruturais
for (const tag of ["main", "nav", "header", "aside", "table"]) {
  const open = (html.match(new RegExp(`<${tag}[\\s>]`, "g")) || []).length;
  const close = (html.match(new RegExp(`</${tag}>`, "g")) || []).length;
  check(`tags <${tag}> balanceadas (${open}/${close})`, open === close);
}

if (failures) { console.error(`\n${failures} falha(s)`); process.exit(1); }
console.log("\nSmoke test da Web GUI: 100% OK");
