# PROGRESS.md — Registro de adições ao projeto Colmeia

> Atualizado em 2026-11-20. Este arquivo lista tudo que foi adicionado/alterado,
> em ordem cronológica, com referências aos artefatos.

## 2026-11-20 — Web GUI v2 (frontend completo), automação de testes e docs

### Web GUI (`web/index.html`, 146 KB, arquivo único, zero dependências)
- **Base**: o HTML anexado (dashboard lightweight) passou a ser a base exata do
  frontend, preservado e estendido (não reescrito).
- **Telas** (12 no total): Visão Geral, Repositórios, Sync, Slice, Merge,
  Segurança, Docs, Grafo, **Branches** (nova), PRs, Anomalias, **Testes** (nova).
- **Tela Branches** (grafo de commits, estilo das referências):
  lanes `main`/`feature/*`/`bugfix/*`/`hotfix/*`/`release/*`, arestas de merge
  curvas, tag de release, animação de desenho progressivo, hover com tooltip,
  clique abre drawer do commit, seletor de 4 repositórios, painel "O que
  aconteceu", convenções de nomenclatura, tabela de comandos e fluxo de 7 passos.
- **Tela Testes**: checklist de automação de 24 pontos (cards com lâmpadas de
  status), botão "Rodar suíte" (simulação com stagger) e exportação de
  relatório JSON.
- **Drawer de repositório com abas**: Resumo (timeline de commits), Syncs
  (sparkline + auto-fetch/mirror), Findings (CVEs por repo), Docs.
- **Grafo da frota**: barra de filtros por cluster (8 chips com cor), sliders de
  profundidade e nós visíveis em tela.
- **Menu hambúrguer**: abre/fecha a navegação como drawer lateral com física de
  mola em ≤ 1180 px; fecha por scrim, Esc ou troca de tela. **Bug corrigido**:
  em 901–1180 px os rótulos sumiam (faltava anular as regras do breakpoint antigo).
- **Estúdio no mobile**: virou bottom-sheet com botão "Estúdio" no header
  (antes era inacessível em ≤ 900 px).
- **Temas**: 33 ao total — 4 base (escuro/claro/sépia/neumorph), 5 retrô
  (Scherzo, Antique, Ice, Onyx, BeBox, System 7), 5 energia (Zeus, Freedom,
  Arwen, Pay First, Yamata) e **18 cyberpunk** (Neon Magenta, Cyan Blue,
  Violet Nebula, Toxic Lime, Noir Black, Steel Gray, Chrome, Warm Amber,
  Tokyo Neon, Midnight Mint, Arasaka Red, Militech Amber, Kiroshi Cyan,
  Afterlife, Glitch Amber, Hazard Frame, Desaturated Dusk, Synthwave Grid).
  Cada tema tem fonte combinada (Orbitron, Audiowide, Oxanium, Bruno Ace SC,
  Share Tech Mono — Google Fonts não bloqueante com fallback offline).
- **Motion design**: ripple nos botões, sweep na troca de tema, sheen animado
  nos cards cyberpunk, glow neon, stagger de listas, drawer com mola — tudo
  controlado pelo multiplicador `--motion` existente (respeita
  `prefers-reduced-motion`).

### Automação de testes
- `web/smoke-test.mjs`: 34 verificações determinísticas da Web GUI (telas, 24
  checkpoints, ≥ 15 temas cyberpunk, sintaxe JS, leveza, tags balanceadas).
- `tests/repos_dir_test.rs`: 4 testes de integração usando o diretório real
  `~/Ubuntu/home/Documents/repos` (override via `COLMEIA_REPOS_TEST_DIR`):
  - descoberta de repositórios git (com limite de profundidade);
  - leitura de HEAD/branches/log **sem mutação**;
  - registro no `Store` SQLite **temporário**;
  - clone em `tempdir` com branch + commit de teste (manipulação permitida),
    garantindo o original intocado.
  - Contrato dos testes: nunca deletar; só ler os originais; escrever só em
    temporário.
- CI (`.gitea/workflows/ci.yaml`): novo job `web-ui` (smoke test + HTTP 200).
- `justfile`: recipes `test-web` e `test-repos`.
- `package.json` + `scripts/dev-server.mjs`: preview estático com Node puro
  (zero deps) — `npm run dev` / `npm run test:web`.

### Docs e processo
- `docs/` — pasta de decisões de arquitetura (ADRs) e levantamento de features.
- `docs/FEATURES-PENDENTES.md` — gap F01–F55 + milestones M0–M9 vs. código atual.
- `PROGRESS.md` — este arquivo.

### Decisões relevantes
- Frontend permanece **server-agnostic**: o mesmo `web/index.html` é servido
  pelo daemon Rust (`include_str!`) e pelo dev-server Node; dados vivos vêm de
  `/events` (SSE) — o restante é estado local com seed determinística.
- Fontes externas são **progressive enhancement** (media-swap), nunca obrigatórias.

## 2026-11-20 (2ª rodada) — Tela Explorar: diretório real de repos navegável

### API read-only no dev-server (`scripts/dev-server.mjs`)
- `GET /api/tree?path=<rel>` — lista 1 nível: pastas, repos git (`.git` dir ou
  arquivo) e contagem de arquivos soltos; lazy, sem limite de profundidade
  hard-coded (a UI expande sob demanda).
- `GET /api/repo?path=<rel>` — HEAD, branch atual, branches locais, último
  commit (assunto/autor/ts), remote origin e dirty count — git read-only com
  timeout de 6 s.
- Guardas: traversal fora do root → 403; método ≠ GET → 405; não-repo → 400;
  inexistente → 404. Root configurável via `COLMEIA_REPOS_ROOT`.
- `cache-control: no-store` em tudo (fim do cache stale do Electron no dev).
- ADR: `docs/adr/0006-api-readonly-exploracao.md`.

### Tela Explorar (`web/index.html` → ~157 KB, ainda arquivo único)
- Nova entrada "Explorar" na navegação (ícone de pasta), entre Repositórios e
  Sync; estilo da referência Riter: árvore lateral com caret animado, badge de
  contagem por pasta, linhas-guia de indentação e stagger de entrada por
  profundidade (`--d`).
- Painel direito com detalhe do repo selecionado: HEAD, branch, nº de branches,
  idade do último commit, assunto/autor, chips de branches, remote, dirty count
  e **"Importar para a frota"** (adiciona ao `state.repos` com undo, sem tocar
  no filesystem).
- Botão "Importar visíveis": importa em lote (4 concorrentes) todos os repos
  já carregados na árvore, sem duplicatas (por nome ou remote).
- Fallback elegante: sem API (ex.: daemon Rust estático), mostra aviso
  "API indisponível — rode `npm run dev`" com botão de retry.
- Descoberta real: ~40 repos na raiz; `projects/` mista com 17+ repos
  (colmeia-app é pasta comum lá, não é repo git).

### Automação de testes
- `scripts/api-test.mjs` (`npm run test:api`, recipe `just test-api`): 18
  verificações contra o diretório real — inclui HEAD hex, subnível projects/ e
  os 4 casos de segurança da API. **100% OK.**
- Smoke test: +5 verificações da tela Explorar (SCREENS, ações `fs-*`,
  endpoints, ícones). **100% OK (39 no total).**

### Fixes da validação em browser (diretório real)
- **Timeout do git 6 s → 25 s**: o mount `~/Ubuntu/home` é lento
  (~15 s para ler refs de um monorepo); comandos que leem refs/objetos
  estouravam o timeout enquanto os que leem arquivos pontuais (HEAD, remote)
  passavam. Adicionado `GIT_OPTIONAL_LOCKS=0` (read-only puro).
- **Split do `git log` corrigido**: separador NUL gravado como string vazia
  fazia `split("")` porcaractere (`subject:"d"`); agora `String.fromCharCode(0)`.
  Teste reforçado: assunto > 5 chars.
- **Conflito de porta 7100**: servidor de preview antigo (IPv6-only, código em
  memória desatualizado) respondia `localhost` e servia 404 nas rotas `/api`.
  Dev-server agora sobe em **dual-stack** (`--host ::`) e envia
  `cache-control: no-store` (fim de cópia stale no Electron).
- Validado ponta a ponta no browser: raiz (7 pastas · 34 repos) → expandir
  `projects/` → detalhe de `projects/lia` (HEAD, 12 branches, último commit,
  remote, dirty 152) → importar para a frota (18 → 19, dedupe por nome/remote).

## 2026-11-21 — Todas as telas refletem a frota real do diretório

### API
- **`GET /api/scan`** (`scripts/dev-server.mjs`): varredura recursiva só de
  filesystem (sem git — rápida no mount lento), BFS profundidade ≤ 3, teto
  400 repos, skip de `node_modules/target/.git/...`. Retorna `path/name/cluster`.

### Frontend (`web/index.html` → ~162 KB, ainda arquivo único)
- **Motor de frota real**: `realScan()` → `applyRealFleet()` substitui
  `state.repos` pelos repos reais e rederiva findings, PRs, anomalias,
  conflitos, jobs, eventos e policies (`sync-<cluster>` por pasta real).
- **Grafo da frota reconstruído** (`rebuildFleetGraph`): clusters = pastas
  reais (`(raiz)`, `ai-agents`, `ciberseguranca`, `criacao-3d`,
  `IA-Design-Motion`, `ia`, `karpathy-curso`, `projects`); hubs e nós usam
  nomes reais; filtros da tela Grafo acompanham.
- **Branches ligados à frota** (`syncBranchRepos`): seletor usa os 4 primeiros
  repos reais.
- **Visão Geral**: KPI "Repositórios" = número real, chip "diretório real"
  com tooltip do path, jobs e policies reais.
- **Slice/Sync**: commit e labels de fluxo usam o primeiro repo real.
- **Cache localStorage** (`colmeia-repos-real`): boot restaura a frota
  conhecida antes da 1ª render; enriquecimento progressivo (branch/HEAD/
  remote via `/api/repo` em lotes de 2) atualiza as telas ao vivo.
- Sem `/api/scan` a UI cai no seed normalmente (server-agnostic preservado).

### Validação
- Descobertos **80 repos reais** (inclui aninhados em `ai-agents/` etc.).
- Browser: Visão Geral KPI=80 + chip real; Repositórios 80 linhas;
  Branches chips reais; Grafo 8 filtros reais; Slice commit `3a7cc04 · 3dixel`;
  Sync policies por cluster.
- `npm run test:api` 22/22 OK (inclui 4 do `/api/scan`); smoke test 44/44 OK.
- ADR: `docs/adr/0007-frota-real-todas-telas.md`.
- Conflito de porta 7100 recorrente (servidor stale do preview em IPv6-only):
  processo antigo encerrado; dev-server sobe dual-stack (`--host ::`).

## 2026-11-21 — Auditoria de código, validação MVP e refinamento UI/UX

### Auditoria (metodologia "lazy senior dev" — menor mudança completa)
- **Must fix → corrigidos:** crash do tick SSE com frota vazia (guard de
  `state.repos.length`); tabelas Repositórios/Segurança sem rolagem
  horizontal (`overflow-x:auto` + min-width); `gate.rs` sem teste → novo
  `tests/gate_test.rs` (token `ghp_` plantado → finding + SARIF) e recipe
  `test-gate` no justfile.
- **Should fix → corrigidos:** enriquecimento da frota real não renderiza com
  input focado; Enter/Espaço ativam `[data-act]` focado; `:focus-visible`
  global; `aria-current="page"` no nav; `role="img"`+`aria-label` nos 5
  canvases; `document.title` por tela; foco inicial em modais.
- Relatório completo: `docs/auditoria-codigo.md`.

### Responsividade (documentada)
- Breakpoints reais mapeados: ≥1181 px desktop (nav 224 + estúdio 316),
  901–1180 px rail+drawer hambúrguer 250 px, ≤900 px coluna única com nav
  drawer `min(300px,86vw)` e estúdio bottom-sheet 46 vh.
- Verificação empírica: 13 telas sem overflow de página no desktop.
- Limitação conhecida: canvas de branches com largura fixa 920 px rola
  horizontalmente em telas estreitas (sistema de coordenadas).
- Documento: `docs/responsividade.md`.

### Validação contra o MVP
- Matriz F01–F55 cruzada com o código: 14 ✅, 9 🟡, restante ❌.
- Antecipação de bugs registrada (A1–A8), destaque para `git merge -X` sem
  `--no-edit` (risco de travar headless) — mitigação proposta, a aplicar com
  teste antes.
- Documento: `docs/validacao-mvp.md`.

### Testes
- `npm run test:web` 50/50 · `npm run test:api` 22/22 · `cargo test --test
  gate_test` 1/1.

## 2026-11-21 (2ª rodada) — Matriz de features completa: lotes 5–9 + loop de validação

### Lote 5 — F51 cofre de segredos · F25 hooks · F52 redação/fuzz
- **`src/secretbox.rs` — correção de 2 bugs reais de criptografia no Poly1305:**
  1. overflow em debug no carry (`g[4] = h[4] + c - (1 << 26)`) → `.wrapping_sub(1 << 26)`;
  2. soma final `h += s` em limbs de 26 bits **dupla-contava os bits 52+**
     (termos `(s1 << 6)` sem máscara vazavam para o limb seguinte) → reescrita
     estilo donna: empacotamento em 4 words de 32 bits + soma com carry.
  Validado com vetor isolado RFC 8439 §2.5.2 e AEAD §2.8.2.
- **Vault (F51):** `Store::secret_put/get` (tabela `secrets`), salt por store,
  nonce aleatório de 12 B (`rand_bytes` com fallback sha256), AEAD com AAD =
  nome do segredo. CLI `colmeia vault set/get` (env `COLMEIA_PASSPHRASE`).
  Testes: `tests/secretbox_vault_test.rs` — 6 ok.
- **Hooks (F25):** `src/hooks.rs` — instalação de `pre-commit`/`pre-push` com
  gate (sem `colmeia` no PATH → passa com aviso), `run_hook` com timeout e
  kill, hooks dir com fallback Rust puro antes de `git rev-parse`.
  CLI `colmeia hooks-install`. Testes: `tests/hooks_test.rs` — 4 ok.
- **Redação (F52):** `src/redact.rs` — tokens inteiros (`ghp_`, `github_pat_`,
  `sk-`, `AKIA/ASIA`, `xox`, chaves privadas multilinha) e pares chave=valor,
  OnceLock, idempotente. **Fuzz proptest:** `tests/fuzz_parsers_test.rs` — 4 ok
  (sbom::parse, rules::scan, redact idempotente, glob_match).

### Lote 6 — F17 Starlark hermético · F14 merge 3-way estruturado
- **`src/starlark.rs`** (~1.100 linhas): lexer com Indent/Dedent, parser,
  avaliador com GAS (200 mil passos), `def`/`return`/`if`/`load` restrito ao
  diretório do pack (canonicalizado — macOS `/var` vs `/private/var`), métodos
  de string, `len`, builtin `rule()`, coleta com dedup (binding `rules` ou
  varredura). Testes: `tests/starlark_test.rs` — 8 ok.
- **`src/structured.rs`** (F14): merge3 JSON recursivo por chave (conflito →
  marcador com path `$.k`), TOML via conversão toml↔serde_json, subset YAML
  próprio (sem crate nova). Decisão: chave removida de um lado + modificada do
  outro = conflito; removida pelos dois = some. Testes unit: 4 ok.

### Lote 7 — F28 grafo · F30 anomalias · F31 drift · F26/27 docs FTS · F29 layout
- **`src/graph.rs`:** arestas cochange por Jaccard de arquivos commitados na
  janela (`git log --since` + `--name-only`); `GraphView`.
  Testes: `tests/graph_test.rs` — 1 ok (repos reais).
- **`src/anomaly.rs`:** EWMA α=0.3 + MAD; veredito calculado **contra a
  baseline anterior** e com piso `max(mad, |ewma|*0.01, 1e-9)` — o pico não
  pode se diluir; warmup de 5 amostras; z>5 = anomalia. Testes unit: 3 ok.
- **`src/fanout.rs`** (F31): drift de versões via SBOM. **`src/layout.rs`**
  (F29): grade determinística + culling por peso/isolados.
- **Docs engine (F26/27):** `src/docs_engine.rs` — indexação incremental por
  content_hash sha256, O(diff); FTS5 via rusqlite bundled (`doc_fts`,
  `repo_path UNINDEXED` como chave); busca com termos entre aspas.

### Lote 8 — F32–F35 forges · F37 webhooks · F38 hooks out · F39/40 plugins · F53 OTLP
- **`src/forge.rs`:** trait `ForgeAdapter` + GitHub + `GitLabLikeAdapter`
  (gitlab/forgejo, `api_base` configurável); `rate_gate` via
  `Store::rate_consume` (janela 1 h por forge). ureq 2 reporta 403 tanto em
  Status quanto em Transport — ambos varridos. Testes: `tests/forge_test.rs`
  — 9 ok (sinks tiny_http/TcpListener locais).
- **`src/webhooks.rs`** (F37): HMAC-SHA256 hex `sha256=`, comparação em tempo
  constante; `hmac_sha256` tornado público no secretbox.
- **`src/hooks_out.rs`** (F38): `HookTarget::Exec` (env `COLMEIA_HOOK`/
  `COLMEIA_EVENT`, payload no stdin) e `Http` POST; entregas persistidas em
  `hook_deliveries`.
- **`src/plugins.rs`** (F39/40): `PluginHost::spawn` + `request` com allowlist
  (NotAllowed antes de qualquer I/O); drop mata o processo.
- **`src/otlp.rs`** (F53): `emit` lê env a cada chamada (sem OnceLock), envelope
  `resourceLogs` simplificado; teste com sink TCP próprio.

### Lote 9 — F43 firmware T0 · F46 TUI+braille · F47 wizard · F48 self-heal · F49/50 · i18n
- **`src/firmware.rs`** (F43): `contract_t0()`, `CONTRACT_VERSION="t0.1.0"`,
  tiers mínimo/recomendado/máximo, comandos e invariants; CLI
  `colmeia firmware-contract`.
- **`src/braille.rs`** (F46): `render_braille` pura — anel determinístico,
  Bresenham, mapa BRAILLE_MAP 2×4; aba `[3] Grafo` na TUI.
- **Wizard (F47):** `parse_profile_choice` pura; interativo só com
  `stdin().is_terminal()`; `Config::apply_profile` (minimum 1w/96MB,
  recommended 2w/256MB, maximum 4w/512MB).
- **Self-heal (F48):** `self_heal()` com `HealAction{check,action,healed}`
  (data-dir, config-file, db-schema, git-binary); CLI `colmeia doctor --heal`.
- **Packaging (F49):** `packaging/homebrew/colmeia.rb` +
  `packaging/debian/build.sh` (validado com `sh -n`).
- **Docs + i18n (F50):** `docs/getting-started.pt.md` + `.en.md`;
  `src/i18n.rs` com `Lang::detect` (`--lang`/`COLMEIA_LANG`), tabela PT/EN,
  fallback pt. Testes: `tests/lote9_test.rs` — 7 ok.

### Bug encontrado no loop de validação (fora da antecipação A1–A8)
- **LRU de worktrees com timestamps de 1 s** (`worktrees_lru_test` falhando
  intermitente): `touch` no mesmo segundo da inserção empatava `last_used` e a
  vítima do despejo era o worktree errado (o renovado, não o menos usado).
  Correção: `lru_clock()` em `store.rs` = `unix()*1_000_000 + contador atômico
  por processo` → ordem total. Validado com 6 execuções consecutivas estáveis.

### Limpeza
- Apagados `examples/dbg_lease.rs` e `examples/dbg_slice.rs`.
- Contrato de leveza mantido: **zero crates novas** além de `cron`, `chrono`
  (default-features=false) e `miniz_oxide` — já presentes no Cargo.toml.

### Documentação
- `docs/validacao-mvp.md` — matriz F01–F55 revalidada: **51 ✅ · 4 ❌**
  (F15 workbench e F36 auto-merge = próximos incrementos; F41 WASM = "só se
  demanda" no plano, fora de escopo por leveza; F55 governance = processo).
  Bugs A1–A8 com mitigação aplicada e teste que cobre.

### Contagem final de testes (loop de validação concluído)
- `cargo test` (suíte completa Rust): **122/122 ok**, 0 falhas —
  inclui testes unit (38), integração com o diretório real de repos,
  vault, hooks, fuzz proptest, starlark, forge com sinks locais e lote 9.
- `npm run test:web`: **50/50 ok** · `npm run test:api`: **22/22 ok**.
- Bug de regressão corrigido neste loop: LRU de worktrees (empate de
  timestamps de 1 s) — `lru_clock()` monotônico, 6 execuções estáveis.

## 2026-11-21 (3ª rodada) — Nav e Estúdio redesenhados como overlays (referência do usuário)

### Layout (`web/index.html`, ainda arquivo único)
- **Fim das colunas fixas**: o grid era `224px 1fr 316px` (nav sempre visível +
  estúdio sempre visível no desktop). Agora o `<main>` ocupa toda a largura e
  nav e estúdio são **overlays fechados por padrão em todos os breakpoints**,
  abertos só por clique — pedido explícito do usuário.
- **Nav = drawer flutuante estilo da referência anexa**: cartão `.rail`
  (trilho escuro `#13151c`, 54 px, logo hexagonal + ícones brancos com estado
  ativo destacado) + cartão `.npanel` (painel claro 232 px, seções FROTA/
  SISTEMA, itens com contagem — Repositórios 346 — e badges — Merge 1,
  Segurança 3). Fundo fosco contínuo (`backdrop-filter:blur`) une os cartões
  e elimina slivers do conteúdo atrás.
- **Estúdio = painel fixo no canto superior direito** (`top:56px;right:0`,
  336 px), oposto ao hambúrguer no canto superior esquerdo; botão "Estúdio" do
  header agora visível em todos os breakpoints; botão fechar adicionado ao
  topo do painel.
- Mecânica preservada: scrim fecha tudo, Esc fecha tudo, transição de mola
  controlada por `--motion` (respeita `prefers-reduced-motion`), aria-expanded
  no hambúrguer e no botão do estúdio.
- Media queries simplificadas: o drawer é o padrão universal; restaram apenas
  colapsos de colunas (≤1180 px) e ajustes de header (≤900 px).

### Testes
- Smoke test atualizado: 52 verificações (novas — drawer com rail+npanel, sem
  coluna fixa, estúdio fechado por padrão no canto superior direito).
  **52/52 OK**; API **22/22 OK**.
- Validação visual no browser: drawer aberto fiel à referência; estado padrão
  com ambos fora da tela (transform medido); estúdio ancorado à direita.
- `docs/responsividade.md` reescrito para o novo modelo de overlays.

## 2026-11-21 (4ª rodada) — Fix do scrim, rail persistente, Estúdio por último e GitFlow

### Bug crítico corrigido: scrim cobrindo os cliques
- **Causa-raiz:** `<nav>` e `<aside class="studio">` viviam dentro de `.app`
  (`position:relative;z-index:1` → contexto de empilhamento). O z-index 25 do
  nav e 24 do estúdio valiam apenas DENTRO desse contexto, então o scrim
  (z-index 19, irmão de `.app` na raiz) pintava por cima de tudo — cliques no
  menu caíam no scrim. Confirmado com `elementFromPoint` (hit = `.scrim show`).
- **Correção:** nav e estúdio movidos para fora de `.app` no DOM (só o
  `<main>` ficou). Header subiu para `z-index:26` (sempre clicável). Smoke
  test ganhou verificação estrutural da ordem do DOM.

### Menu hambúrguer fechado = trilho de ícones sempre visível (referência)
- **Rail (`nav .rail`) agora é persistente** em todos os breakpoints: trilho
  escuro com logo + ícones das telas, sempre clicável (atalho direto).
- **Painel expande do próprio trilho** (`nav.open .npanel`): desliza da
  posição do rail, sem criar barra nova. `nav` fechado tem
  `width:62px;overflow:hidden` + `translateX(calc(-100% - 62px))` no painel —
  corrige o vazamento do conteúdo direito do painel (Fechar/contagens/
  badges) sobre o rail (bug visual da rodada anterior).
- `.app{padding-left:74px}` para o conteúdo respeitar o rail.

### Estúdio como último item do header
- Botão "Estúdio" movido para depois de "＋ Novo projeto" (último ícone da
  parte superior direita), visível em todos os breakpoints.

### GitFlow estilo GitKraken nos projetos (guideline: imagem anexa)
- **API nova `GET /api/log?path=<rel>&n=N`** (dev-server): `git log
  --topo-order` com pais e refs (%H %P %s %an %ct %D); read-only, anti-
  traversal herdado, limite n 10–400; 400 fora de repo.
- **Aba "GitFlow" no drawer de repo**: grafo de lanes estilo GitKraken —
  canvas com linhas verticais por lane, dots coloridos por convenção
  (main verde, feature violeta, bugfix âmbar, hotfix vermelho, release azul),
  arestas de merge em curva Bézier da lane do filho até a lane do pai,
  anel no HEAD; lista de commits ao lado (sha, assunto, autor, idade, tags de
  ref). Canvas `position:sticky` no scroll compartilhado com a lista.
- **Algoritmo de lanes** (`gfLayout`): commit ocupa a lane que o esperava;
  1º pai continua na mesma lane; demais pais (merge) reutilizam lane livre
  ou abrem nova. Cache por repo (`GF_CACHE`).
- Drawer fica amplo (`.drawer.wide`, 700 px) na aba GitFlow.
- Dados reais validados: projects/lia → 48 commits, 4 lanes, 3 merges com
  curvas visíveis.

### Testes
- Smoke: 58 verificações (novas — nav/estúdio fora do .app, painel sem
  interceptação de cliques, Estúdio último do header, GitFlow). **100% OK**.
- API: 27 verificações (novas — /api/log ok, sha/subject, parents array,
  limite n, 400 em não-repo). **100% OK**.
- Conflito de porta 7100 recorrente resolvido de novo: processo node stale
  (PID antigo em `[::1]:7100`) derrubado; novo sobe dual-stack.

### Docs
- `docs/responsividade.md` atualizado (rail persistente + painel expansível,
  modelo de padding do conteúdo).

## 2026-11-22 — Automação GitHub CLI + Hugging Face (detecção de CI determinística)

Pedido: identificar na documentação do GitHub CLI métodos de automatizar
tarefas (detecção de falhas de CI, otimização de código, inserção de
features) com o máximo de determinismo, utilizáveis neste software, com base
em docs.github.com, documentação de Webhooks, Hugging Face Inference
Providers e padrões de outros repositórios de automação de CI.

### Novo módulo `src/ghci.rs` — wrapper testável da GitHub CLI
- `GhCli` encapsula `gh` via `std::process::Command` (zero crates novas —
  contrato de leveza). Métodos espelham a documentação oficial:
  - `detect_failures` → `gh run list --repo R --status failure --limit N
    --json databaseId,workflowName,headBranch,conclusion,createdAt,url`;
  - `failure_log` → `gh run view <id> --log-failed` (só passos falhos,
    truncado em 8 000 chars — padrão gh-fix-ci);
  - `rerun_failed` → `gh run rerun <id> --failed` (triagem flaky; **só**
    via flag explícita `--rerun`, nunca automático);
  - `repo_slug_from_remote` extrai `owner/repo` de remotes https/ssh.
- Erros limpos: `GhciError::{NotInstalled, Exit{code,stderr}, Parse}`.
- `ci_scan` (orquestração): slugs ordenados+deduplicados, rate budget
  `github-cli` 60/hora via `store.rate_consume`, evento `ci_failure` por
  achado, relatório `{findings, deduped, rate_limited}`.

### Novo módulo `src/hf.rs` — Hugging Face Inference (opt-in)
- Endpoint OpenAI-compatível `https://router.huggingface.co/v1`
  (docs HF). Sem `HF_TOKEN` → cliente ausente + erro limpo; modelo default
  `Qwen/Qwen2.5-Coder-32B-Instruct` (override `HF_MODEL`).
- Determinismo: `temperature: 0`, `max_tokens: 512`, prompt de sistema com
  formato fixo. IA **resume e sugere** — nunca edita nem age sozinha.

### Dedup idempotente — `ci_runs` (schema.sql + migration)
- Tabela `ci_runs(run_id PK, repo, workflow, conclusion, seen_at)`;
  `Store::ci_seen` / `Store::ci_mark` (INSERT OR IGNORE). Um run de CI é
  reportado no máximo uma vez, mesmo em loop de scan — anti-loop
  determinístico (mesma filosofia da mitigação A6).

### CLI
- `colmeia ci-scan [--repo owner/repo] [--limit N] [--explain] [--rerun]`.
  Sem `--repo`, varre o registry e usa o remote `origin` de cada repo para
  achar slugs GitHub. `--explain` exige HF_TOKEN (avisa e segue sem).

### Testes
- `tests/ghci_test.rs` — 11 testes: parse JSON canônico, flags exatas,
  truncamento, rerun `--failed`, exit≠0→`Exit`, gh ausente→`NotInstalled`,
  idempotência por run_id, evento `ci_failure`, ordenação/dedup de slugs,
  cliente HF contra sink TCP local (path `/v1/chat/completions`,
  `temperature: 0`, `Authorization: Bearer`, `max_tokens: 512`).
- 2 testes unitários em `src/ghci.rs` (slug de remote, truncate UTF-8).
- Correção de corrida: os 8 testes que mutam env (PATH/FAKE_GH_*) agora
  serializam via `ENV_LOCK` — threads paralelas trocavam PATH uma da outra
  (ENOENT no spawn do gh fake e contagens erradas de chamadas).
- Suíte completa: **136 testes Rust, 0 falhas**. Web smoke 58/58, API 27/27.

### Docs
- `docs/github-cli-automacao.md` (novo): tabela gh CLI→feature, decisões D1
  (encapsular CLI vs REST) e D2 (mutação nunca implícita), webhooks
  determinísticos (`workflow_run`/`workflow_job`, `X-GitHub-Delivery` dedup,
  anti-recursão — apoiado em `webhooks.rs`/`hooks_out.rs` existentes),
  padrões de outros repos (extrato de log, flaky vs real, plano-antes-agar),
  roadmap event-driven.

## 2026-11-23 — Release multi-plataforma + loop de polimento premium (8h)

### Release builds (macOS, Windows, Linux)
- `.github/workflows/release.yml`: validação (testes Rust + smoke web + API
  contra fixture sintética) em 3 SOs; build release em 4 alvos
  (macOS x86_64/arm64, Windows x86_64, Linux x86_64); GitHub Release com
  os pacotes ao publicar tag `v*`. Disparo manual via `workflow_dispatch`.
- macOS local: `dist/colmeia-macos-arm64` (4,8 MB) buildado e validado
  (`--help`/`--version` executam; mount não permite exec — testado via /tmp).
- Portabilidade dos testes: `ghci_test` isolado em `cfg(unix)`; mesma
  serialização de PATH (ENV_LOCK) aplicada em `hooks_test` (corrida de PATH
  entre testes paralelos quebrava `sleep` no CI → exit 127); `forge_test` e
  `hooks_test` unix-only (scripts shell); `repos_dir_test` faz skip
  gracioso fora da máquina dev (exige o diretório real de repos).

### Polimento premium do dashboard (referência: imagem do usuário)
- p01: sombras em camadas por tema (`--shadow-card`) + raio 14px
  (fix: `applyThemeVars` resetava `--r` para 12 via fallback).
- p02: item ativo do rail em pílula de acento (bg 30% + anel + glow).
- p03: `-webkit-font-smoothing:antialiased` + `text-rendering:
  optimizeLegibility` + títulos de card com peso/tracking refinados.
- p04: tabela sem borda na última linha; foco de inputs/selects com anel
  de acento; toast elevado.
- Verificação via DOM no navegador embutido (raio, sombras e rail ativo
  confirmados por computed styles).
- Fila completa em `docs/polish-queue.md` (p05–p15) com critérios de aceite,
  regras do loop e log de iterações.

### Loop de polimento 8h (automação)
- `automation_0862abff`: cron `23 * * * *` (America/Sao_Paulo), modelo
  k3-agent, execução no workspace colmeia, timeout 45 min/run.
  Cada run: pega o próximo item da fila, implementa, valida, commita,
  envia e se auto-desativa após 8h ou ao esgotar a fila.
  Nota: cada execução consuma créditos de conversação (aviso dado ao dono).

### Testes
- Local: web 58/58, API 27/27, hooks 4/4, ghci 11/11, forge 4/4,
  repos_dir 4/4. CI: acompanhado em 3 rodadas até ficar verde nos 3 SOs.
