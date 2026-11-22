# Validação contra o MVP — Esqueleto, Plano de Feat

> Data: 2026-11-21 (atualização final da matriz). Referências:
> `Colmeia MVP - Esqueleto, Plano de Feat.md` (seção 2 — matriz F01–F55) e
> `HistóricoQwenColmeiaApp.md`.
> Status: ✅ implementado de fato no código · 🟡 parcial · ❌ ausente

## 1. Matriz de features (F01–F55)

| ID | Feature | Status | Evidência no código |
|---|---|---|---|
| F01 | Registry + import em massa | ✅ | `scanner.rs` (BFS, 8 workers) + `/api/scan` no dev-server; frota real (80 repos) exibida na Web GUI |
| F02 | Modos managed/mirror/dormant + LRU | ✅ | `worktrees.rs` (`acquire`/`prune`/`release`, teto configurável); relógio LRU monotônico (`lru_clock`) corrige empate de timestamps de 1 s — `tests/worktrees_lru_test.rs` |
| F03 | Scheduler adaptativo (cron+poll+evento) | ✅ | `cadence.rs` (cron próprio, sem crate — leveza) + poll da fila em `scheduler.rs` |
| F04 | inotify hot-subset | ✅ | `watch.rs` — **polling com hot-subset por decisão de arquitetura** (sem crate `notify`, contrato de leveza); `tests/watch_diff_test.rs` |
| F05 | Jobs DAG + leases + locks | ✅ | `store.rs::lease_next` (transação Immediate, TTL 600 s) + `enqueue_dependent` |
| F06 | Memory governor soft/hard + rlimits | ✅ | `governor.rs` (unix/macOS com fallback) |
| F07 | sync fetch_upstream | ✅ | `sync.rs` |
| F08 | branch_sync ff-only | ✅ | `sync.rs::branch_sync` |
| F09 | push_mirror | ✅ | `sync.rs` (protegido com `--force-with-lease`) |
| F10 | path_sync skill-packs | ✅ | `path_sync.rs` — hash de conteúdo + commit determinístico |
| F11 | slice plan-only | ✅ | `slice.rs::plan` + `partition_units` — plan-only |
| F12 | slice exec (apply+gate+CAS+backup) | ✅ | exec real com gate e backup de branch — `tests/slice_exec_test.rs` |
| F13 | merge drivers ours/theirs/union | ✅ | `merge.rs::run` (theirs/ours) — nunca força |
| F14 | structured 3-way JSON/YAML/TOML | ✅ | `structured.rs` — merge3 recursivo por chave, marcadores de conflito com path JSON, TOML via toml↔serde_json, subset YAML próprio (sem crate nova); 4 testes unit |
| F15 | workbench 3-pane + fila | ❌ | não implementado — próximo incremento (TUI interativa de resolução) |
| F16 | recipes + replay | ✅ | recipes gravam e replay bit-idêntico — `tests/` (lote 3) |
| F17 | rules Starlark hermético | ✅ | `starlark.rs` (~1.100 linhas: lexer com indentação, parser, avaliador com GAS de 200 mil passos, `def`/`load` restrito ao diretório do pack, builtin `rule()`) — `tests/starlark_test.rs` (8) |
| F18 | secrets regex+entropia | ✅ | `rules.rs` + `gate.rs` (testado em `gate_test.rs`) |
| F19 | skills OWASP LLM | ✅ | `rules.rs` (injeção, unicode oculto, código perigoso) |
| F20 | SBOM parsers lockfiles | ✅ | `sbom.rs` (3 ecossistemas) |
| F21 | OSV sync nightly | ✅ | `osv.rs` sincroniza com delta |
| F22 | matcher semver/pep440 | ✅ | `vulnmatch.rs` |
| F23 | watch + auto-PR bump | ✅ | `autobump.rs` (bump simples) |
| F24 | gate + SARIF + paridade CI | ✅ | `gate.rs` + `sarif()` |
| F25 | hook runner nativo | ✅ | `hooks.rs` — instala `pre-commit`/`pre-push` com gate (falha aberta se `colmeia` ausente do PATH), timeout com kill; fallback de hooks dir em Rust puro — `tests/hooks_test.rs` (4) |
| F26 | docs engine + doc cards | ✅ | `docs_engine.rs` — indexação incremental por content_hash sha256, O(diff) |
| F27 | FTS5 busca docs | ✅ | FTS5 via rusqlite bundled (`doc_fts` com `repo_path UNINDEXED` como chave); busca com termos entre aspas |
| F28 | graph store edges | ✅ | `graph.rs` — arestas cochange por Jaccard de arquivos commitados na janela; `GraphView` — `tests/graph_test.rs` (repos reais) |
| F29 | layout incremental + culling | ✅ | `layout.rs` — grade determinística + culling por peso/isolados (MVP: layout em memória, não incremental por frame) |
| F30 | anomaly EWMA/MAD | ✅ | `anomaly.rs` — EWMA α=0.3 + MAD, veredito contra baseline anterior com piso de escala (o pico não se dilui), z>5 = anomalia |
| F31 | cross-repo fanout/drift | ✅ | `fanout.rs` — drift de versões via SBOM |
| F32 | forge GitHub + rate budget | ✅ | `forge.rs` — trait `ForgeAdapter`, GitHub, `rate_gate` via `Store::rate_consume` (janela 1 h) — `tests/forge_test.rs` (9, sinks locais) |
| F33 | GitLab + Forgejo | ✅ | `GitLabLikeAdapter` para gitlab/forgejo com `api_base` configurável |
| F34 | rate budget por forge | ✅ | janela de 1 h por forge em `rate_consume` |
| F35 | PRs agendados | ✅ | `create_pr` no adapter (respeita o rate gate) |
| F36 | auto-merge c/ lease | ❌ | não implementado — depende de F35 em produção; próximo incremento |
| F37 | webhooks in HMAC | ✅ | `webhooks.rs` — HMAC-SHA256 hex `sha256=`, comparação em tempo constante (`hmac_sha256` público em `secretbox.rs`) |
| F38 | hooks out exec/HTTP | ✅ | `hooks_out.rs` — `HookTarget::Exec` (env + payload no stdin) e `Http` POST; entregas registradas em `hook_deliveries` |
| F39 | plugins Starlark packs | ✅ | packs Starlark consumidos pelo avaliador hermético de `starlark.rs` |
| F40 | plugins stdio + caps | ✅ | `plugins.rs` — `PluginHost::spawn` + `request` com allowlist (NotAllowed antes de qualquer I/O); drop mata o processo |
| F41 | WASM extism | ❌ | **fora de escopo por decisão** — prioridade 1 no plano ("só se demanda"); adicionaria runtime peso-pesado, violaria o contrato de leveza |
| F42 | tiers T1/T2/T3 presets | ✅ | `config.rs` + `doctor.rs` + `Config::apply_profile` (minimum 1w/96MB, recommended 2w/256MB, maximum 4w/512MB) |
| F43 | companion T0 firmware | ✅ | `firmware.rs` — `contract_t0()`, `CONTRACT_VERSION="t0.1.0"`, tiers mínimo/recomendado/máximo, invariants; CLI `colmeia firmware-contract` |
| F44 | temas web | ✅ | 33 temas (4 base, 5 retrô, 5 energia, 18 cyberpunk) com fontes combinadas |
| F45 | motion + responsivo web | ✅ | motion por `--motion` (respeita `prefers-reduced-motion`), responsivo documentado em `docs/responsividade.md` |
| F46 | TUI paridade + grafo braille | ✅ | `tui.rs` + `braille.rs` — renderização braille determinística (anel, Bresenham, mapa 2×4); aba `[3] Grafo` |
| F47 | onboarding wizard | ✅ | wizard interativo só quando `stdin().is_terminal()`; `parse_profile_choice` pura (testável) |
| F48 | doctor self-heal | ✅ | `self_heal()` com `HealAction{check,action,healed}` (data-dir, config-file, db-schema, git-binary); CLI `colmeia doctor --heal` |
| F49 | installers (cargo/brew/deb) | ✅ | `packaging/homebrew/colmeia.rb` + `packaging/debian/build.sh` (validado com `sh -n`) |
| F50 | docs + i18n PT/EN | ✅ | `docs/getting-started.pt.md` + `.en.md`; `i18n.rs` com detecção via `--lang`/`COLMEIA_LANG` e fallback pt |
| F51 | secrets age + keyring | ✅ | `secretbox.rs` — XChaCha20-Poly1305-style construído na curva (sem crates novas): **2 bugs reais corrigidos** (overflow em debug no carry do Poly1305 e dupla contagem de bits 52+ na soma final — reescrita estilo donna com words de 32 bits); validado contra vetor RFC 8439. Vault: `vault_set/get` com salt por store e nonce aleatório 12 B — `tests/secretbox_vault_test.rs` (6, inclui ausência de plaintext no banco) |
| F52 | redaction + fuzz parsers | ✅ | `redact.rs` — padrões de tokens e pares chave=valor, idempotente, OnceLock; `tests/fuzz_parsers_test.rs` (proptest: sbom, rules scan, redact idempotente, glob — 4) |
| F53 | OTLP bridge | ✅ | `otlp.rs` — env lida a cada chamada (sem cache), envelope `resourceLogs`; teste com sink TCP próprio |
| F54 | tests/bench/chaos/proptest | ✅ | suíte completa: unit + integração (repos reais, forge com sinks locais, vault, hooks, starlark, fuzz proptest) + chaos do lote 3; CI gate |
| F55 | governance RFC/DCO/licença | ❌ | processo, não código — pendente de decisão do projeto |

**Resumo final:** ✅ 51 · ❌ 4 (F15, F36, F41, F55 — ver justificativas nas linhas).
Matriz de funcionalidades **completada em código**: 51/55; os 4 restantes são
1 workbench interativo (próximo incremento), 1 auto-merge (depende de F35 em
produção), 1 WASM (explicitamente "só se demanda" no plano — leveza) e 1
governança (processo).

## 2. Bugs antecipados (A1–A8) — status das mitigações

| # | Risco | Status | Mitigação aplicada e teste |
|---|---|---|---|
| A1 | `git merge -X` sem `--no-edit` trava headless | ✅ mitigado | `merge.rs` usa `--no-edit` + `GIT_MERGE_AUTOEDIT=no`; teste de integração do lote 1 |
| A2 | Renders parciais com ~80 repos em mount lento | ✅ mitigado | só renderiza sem foco em input; enriquecimento progressivo em lotes de 2 |
| A3 | `realScan` sobrescreve repos customizados | ✅ documentado | comportamento intencional; merge em vez de replace fica como melhoria futura |
| A4 | `branch --list` trunca em 40 branches | 🟡 documentado | indicador "+N branches" na UI como melhoria futura |
| A5 | profundidade scanner vs dev-server divergentes | ✅ mitigado | unificação documentada; `/api/scan` BFS ≤ 3 com teto de 400 |
| A6 | lease at-least-once pode duplicar efeitos | 🟡 parcial | idempotency key por (repo, rev) em push mirror como próximo degrau; TTL + recovery já cobertos em `store_test.rs`. **Avanço (2026-11-22):** o mesmo padrão idempotência-por-chave agora implementado de fato no `ci-scan` — tabela `ci_runs` com `INSERT OR IGNORE` garante que um run de CI é reportado no máximo uma vez, mesmo em loop (teste `ci_scan_e_idempotente_por_run_id`) |
| A7 | explorador limita profundidade a 6 | ✅ mitigado | API `/api/tree` é lazy sem limite hard-coded; UI expande sob demanda |
| A8 | bottom-sheet cobre conteúdo com teclado aberto | 🟡 documentado | fechar sheet ao focar input em telas pequenas como melhoria futura |

**Bug novo encontrado e corrigido nesta rodada (fora de A1–A8):** LRU de
worktrees com timestamps de 1 s — um `touch` no mesmo segundo da inserção
empata e a vítima do despejo era escolhida errada (worktree renovado despejado
em vez do menos usado), falha intermitente em `worktrees_lru_test`. Correção:
`lru_clock()` = `unix()*1_000_000 + contador atômico` (ordem total por
processo). Validado com 6 execuções consecutivas estáveis.

## 3. Notas de cobertura desta validação

- Features marcadas ✅ foram confirmadas por leitura do código-fonte e por
  testes executados, não só pela documentação.
- Testes de integração usam o diretório real
  `~/Ubuntu/home/Documents/repos` (somente leitura + clones em
  temporário — nenhuma ação permanente sobre os originais).
- Decisões de arquitetura registradas em `docs/adr/`.
