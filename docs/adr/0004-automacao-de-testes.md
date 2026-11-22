# ADR 0004 — Automação de testes (smoke Web GUI + diretório real de repos)

- Status: Aceito
- Data: 2026-11-20

## Contexto

Pedido: adicionar automação de testes, baseada nos repositórios reais do
diretório `~/Ubuntu/home/Documents/repos`, **sem ações permanentes
como deleção** — com permissão para mover e manipular projetos.

## Decisão

Duas camadas:

1. **`web/smoke-test.mjs`** (rápido, toda CI): 34 verificações determinísticas
   do `web/index.html` — telas presentes, 24 checkpoints de testes, ≥ 15 temas
   cyberpunk, sintaxe JS, ausência de `<script src>`, tamanho, tags balanceadas.
2. **`tests/repos_dir_test.rs`** (integração, local): 4 testes sobre o diretório
   real (override por `COLMEIA_REPOS_TEST_DIR`):
   - descoberta determinística de repos git (limite de profundidade 3);
   - leitura de HEAD/branches/log **read-only**;
   - registro no `Store` SQLite em `tempdir`;
   - **manipulação só em clone** (`file://` → `tempdir`): branch + commit de
     teste; assert de que o original não mudou.

### Contrato dos testes com dados reais

- **Nunca deletar** conteúdo do diretório real.
- Somente leitura nos originais (HEAD, branches, log, status).
- Escrita/manipulação exclusivamente em `tempdir` (auto-limpeza ≠ deleção de
  projeto do usuário).

## Alternativas consideradas

- **Operar diretamente nos originais** (branch de teste no repo real):
  rejeitada — qualquer falha no meio deixaria lixo permanente.
- **Repos sintéticos apenas**: mantidos (já existem em `tests/git_test.rs`), mas
  não substituem validar contra frotas reais.

## Consequências

- CI roda só o smoke; o teste de integração é local (`just test-repos`).
- Descoberta limita profundidade (3) e quantidade para caber no mount lento.
