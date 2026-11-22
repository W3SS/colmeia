# Auditoria de Código — Colmeia

> Data: 2026-11-21 · Método: revisão estilo "lazy senior dev" (menor mudança que
> resolve o problema por completo) · Escopo: `src/*.rs` (daemon Rust) e
> `web/index.html` (Web GUI) · Autor: auditoria Kimi Work

## 1. O que o código faz (resumo por módulo)

### Rust (`src/`, ~2.100 linhas)

| Módulo | Responsabilidade | Observação de audit |
|---|---|---|
| `store.rs` | Fila SQLite (jobs) com **lease em transação `Immediate`** | Lease com TTL de 600 s e re-aquisição de jobs `running` expirados — semântica at-least-once correta; idempotência fica a cargo dos drivers |
| `scheduler.rs` | Poll da fila + despacho para drivers | Sem cron/adaptação — fila pura (ver `docs/validacao-mvp.md`, F03) |
| `scanner.rs` | Descoberta de repositórios em diretório raiz | BFS com **8 workers** (`Vec::with_capacity(8)` + `thread::spawn`); adequado para CLI local, sem limite explícito de profundidade |
| `governor.rs` | Limites de CPU/IO por job | `#[cfg(unix)]` com ramificação `target_os = "macos"`; fallback fora de unix não quebra o build |
| `gate.rs` | Varredura de segredos + relatório SARIF | Padrões de token (ex.: `ghp_`), skip de arquivos > 1 MB |
| `sync.rs` | Fetch, espelhamento de branches (`branch_sync`), push mirror | Operações destrutivas protegidas por `--force-with-lease` onde cabível |
| `merge.rs` | Merge drivers (`-X theirs` / `-X ours`) | ⚠️ sem `--no-edit` — ver "Antecipação de bugs" em `validacao-mvp.md` |
| `slice.rs` | Particionamento determinístico (`partition_units`) | Plan-only no Rust — execução real é gap (F12) |
| `rules.rs` | Regras: segredos, injeção, unicode | Coberto por `tests/rules_test.rs` |
| `sbom.rs` | SBOM para 3 ecossistemas | Coberto por `tests/sbom_test.rs` |
| `osv.rs` | Sync de advisories OSV | Matcher ainda não existe (F21/F22) |
| `web.rs` | API HTTP + SSE (`/api/repos`, `/api/scan`, `/events`) | ~200 linhas, rotas enxutas |
| `tui.rs` | TUI esqueleto (~107 linhas) | 2 abas, sem grafo braille (F46) |
| `doctor.rs`, `config.rs`, `cli.rs`, `events.rs`, `main.rs`, `lib.rs` | Infra de CLI/config/eventos | Sólidos, sem achados |

### Web GUI (`web/index.html`, ~2.260 linhas, arquivo único)

SPA sem framework: estado global + `render()` por tela (13 telas), sistema de
temas via CSS custom properties (33 temas), motion via `--motion`/
`--density`/`--base`/`--micro`, frota real lida do diretório via
`/api/scan` + enriquecimento progressivo por `/api/repo`.

## 2. Findings — Must fix (corrigidos nesta rodada)

| # | Problema | Impacto | Correção |
|---|---|---|---|
| M1 | Tick do SSE simulado lia `state.repos[0]` sem guard | **Crash** (`undefined.dirty`) se a frota estivesse vazia no momento do tick | Guard `state.repos.length` antes de derivar o status |
| M2 | Tabelas de Repositórios e Segurança sem contêiner de rolagem horizontal | Conteúdo transbordava ~28 px no card de findings (viewport desktop) e quebrava layout em telas estreitas | `overflow-x:auto` + `min-width` nas tabelas |
| M3 | `gate.rs` sem nenhum teste — módulo de segurança mais sensível do daemon | Regressão silenciosa em varredura de segredos | Novo `tests/gate_test.rs`: repo temporário com token `ghp_` plantado → `gate::run_repo` registra `secret.token` e emite SARIF; recipe `test-gate` no `justfile`. **Passa** |

## 3. Findings — Should fix (corrigidos nesta rodada)

| # | Problema | Correção |
|---|---|---|
| S1 | `enrichRealRepos` re-renderizava a UI enquanto o usuário digitava/focava campos (80 repos × mount lento = dezenas de renders parciais) | Enriquecimento só renderiza quando nenhum `input/textarea/select` tem foco |
| S2 | Sem suporte de teclado para itens acionáveis (`[data-act]`) | Enter/Espaço no elemento focado disparam a ação; `:focus-visible` global |
| S3 | Navegação sem `aria-current`; canvases sem papel semântico | `aria-current="page"` no nav; `role="img"` + `aria-label` nos 5 canvases (graphCanvas, miniGraph, flowCanvas, donut, branchCanvas) |
| S4 | Título da aba fixo | `document.title` atualiza ao navegar (`Colmeia · <nome da tela>`) |
| S5 | Modal sem foco inicial | Foco vai ao primeiro campo ao abrir modal |

## 4. Nice to have (registrado, não aplicado)

- Nomes de repo na tabela já ganharam ellipsis + `title` (tooltip nativo).
  Um tooltip rico (com path completo e remote) seria o próximo degrau.
- Scanner com 8 threads é aceitável para CLI local; um pool com
  `available_parallelism()` seria mais correto em máquinas muito fracas.
- Cobertura de testes ainda concentrada em scan/store/slice/rules/sbom/git/
  governor/determinism/repos_dir/gate — faltam suites para `sync`, `merge`,
  `web` e a TUI.

## 5. Escopo não auditado em profundidade

- Corpo completo de `tui.rs` (107 linhas — lido parcialmente; é esqueleto).
- Caminho de escrita do event log (auditado por inspeção, não executado).
- Comportamento real de `git merge -X` com conflitos não auto-resolúveis
  (depende de `GIT_EDITOR` do ambiente — ver antecipação de bugs).

## 6. Verificação

- `npm run test:web` — **50 PASS, 100 % OK**
- `npm run test:api` — **22 PASS, 100 % OK**
- `cargo test --test gate_test` — **1 passed**
- Detector de overflow de página rodado nas 13 telas (viewport desktop):
  `docOverflowX=false` em todas.
