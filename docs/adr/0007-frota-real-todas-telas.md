# ADR 0007 — Todas as telas refletem a frota real do diretório

- Status: Aceito
- Data: 2026-11-21

## Contexto

A tela Explorar (ADR 0006) navegava o diretório real, mas o restante da UI
seguia com seed sintético (`fleet/agent-skills-07` etc.). O dono pediu que
**todas** as telas — inclusive Visão Geral — reflitam os projetos de
`~/Ubuntu/home/Documents/repos`.

## Decisão

- Novo endpoint **`GET /api/scan`** no dev-server: varredura recursiva **só de
  filesystem** (sem chamar git → rápida mesmo no mount lento), BFS com
  profundidade ≤ 3 e teto de 400 repos, não descendo dentro de repos. Retorna
  `{path, name, cluster}` onde cluster = primeiro segmento (`(raiz)` para
  repos no nível superior).
- **Motor de frota real no frontend** (`realScan` → `applyRealFleet`):
  substitui `state.repos` pelos repos reais e rederiva tudo que depende de
  nomes: findings, PRs, anomalias, conflitos, jobs, eventos e policies
  (`sync-<cluster>` por pasta real). Grafo da frota (`GNODES/GEDGES`) e
  seletor de Branches (`BRANCH_REPOS`) passam a ser reconstruídos
  (`rebuildFleetGraph`, `syncBranchRepos`).
- **Cache em `localStorage`** (`colmeia-repos-real`): o boot restaura a
  última frota conhecida antes da primeira render (UI útil offline) e o scan
  refresca ao vivo. **Enriquecimento progressivo**: branch/HEAD/remote por
  repo vêm de `/api/repo` em lotes de 2 (o mount é lento), atualizando as
  telas conforme chegam.
- Frontend continua **server-agnostic**: sem `/api/scan`, a UI roda com seed
  normalmente.

## Consequências

- 80 repos reais descobertos (o scan achou repos aninhados em `ai-agents/`,
  `ciberseguranca/` etc. que a listagem lazy da tela Explorar não via).
- KPI "Repositórios" da Visão Geral passa a ser o número real, com chip
  "diretório real"; jobs do painel, policies da Sync, labels do fluxo, commit
  da Slice e clusters do Grafo usam nomes reais.
- `npm run test:api` ganhou 4 verificações do `/api/scan`; smoke test ganhou 5
  do motor de frota real.
