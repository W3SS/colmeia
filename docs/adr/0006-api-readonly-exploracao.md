# ADR 0006 — Exploração do diretório real de repos via API read-only

- Status: Aceito
- Data: 2026-11-20

## Contexto

O dono pediu que o projeto em execução liste e navegue pelas pastas e
repositórios de `~/Ubuntu/home/Documents/repos` (~40 repos no nível
raiz + pasta `projects/` mista com dezenas de repos, PDFs, zips e imagens).
O frontend era 100% seed local. O contrato de leveza proíbe SPA/framework e o
backend Rust (tiny_http) ainda não expõe endpoints de filesystem.

## Decisão

- A API de exploração foi adicionada ao **dev-server** (`scripts/dev-server.mjs`),
  não ao daemon Rust: preview imediato sem tocar em produção.
- **Somente leitura**, espelhando o contrato dos testes (`tests/repos_dir_test.rs`):
  - `GET /api/tree?path=<rel>` — 1 nível por chamada (lazy), separando pastas,
    repos git (`.git` dir **ou arquivo**, p/ worktrees) e contagem de arquivos;
  - `GET /api/repo?path=<rel>` — HEAD, branch atual, branches locais, último
    commit, remote origin e nº de arquivos dirty — tudo via `git` com
    subcomandos read-only e timeout de 6 s.
  - Qualquer outro método → 405; traversal fora do root → 403;
    `COLMEIA_REPOS_ROOT` permite apontar outro diretório.
- **Frontend permanece server-agnostic**: a tela Explorar detecta API ausente
  e mostra aviso com retry em vez de quebrar; quando servida pelo daemon Rust
  (sem `/api/*`), o restante da UI segue funcionando com o seed.
- Dev-server passa a enviar `cache-control: no-store` (evita cópia stale do
  `index.html` no Electron durante iteração).

## Consequências

- `npm run test:api` (`scripts/api-test.mjs`): 18 verificações contra o
  diretório real — raiz, subnível `projects/`, detalhe de repo, HEAD hex,
  e os 4 casos de segurança (traversal 403, não-repo 400, 404, POST 405).
- Smoke test cobre a tela Explorar (SCREENS, ações `fs-*`, lazy load, ícones).
- Importação de repos para a frota é **só no estado local** (localStorage),
  com undo — nada escreve no filesystem real.
