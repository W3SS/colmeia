# Levantamento de features pendentes

Fontes: `Colmeia MVP - Esqueleto, Plano de Feat.md` (matriz F01–F55 + roadmap de
fases) e `HistóricoQwenColmeiaApp.md` (arquitetura, milestones M0–M9, checklist M0).
Estado medido contra o código em `src/` (~2.100 linhas) em 2026-11-20.

**Legenda:** ✅ implementado · 🚧 parcial · ❌ pendente

## Matriz F01–F55 (plano MVP)

| ID | Feature | Estado | O que falta |
|---|---|---|---|
| F01 | Registry + import em massa | 🚧 | `scanner::scan` + registro no store existem; import em massa ≤ 60 s/2.000 repos não validado |
| F02 | Modos managed/mirror/dormant + LRU | 🚧 | modos no schema; LRU de worktrees ❌ |
| F03 | Scheduler adaptativo (cron+poll+evento) | 🚧 | `scheduler::run/run_daemon`; adaptação por carga e cron por policy ❌ |
| F04 | inotify hot-subset | ❌ | — |
| F05 | Jobs DAG + leases + locks | 🚧 | enqueue/lease/finish + dependentes; recovery pós `kill -9` sem teste chaos |
| F06 | Memory governor soft/hard + rlimits | 🚧 | `governor` (RSS/pressão) + rlimits no git; enforcement no scheduler ❌ |
| F07 | sync fetch_upstream | ✅ | `sync::fetch` (básico) |
| F08 | branch_sync ff-only | ✅ | `sync::branch_sync` (básico) |
| F09 | push_mirror | ✅ | `sync::push_mirror` (básico) |
| F10 | path_sync skill-packs | ❌ | — |
| F11 | slice plan-only | ✅ | `slice::plan` + `partition_units` |
| F12 | slice exec (apply+gate+CAS+backup) | ❌ | só o plano existe |
| F13 | merge drivers ours/theirs/union | 🚧 | `merge::run` básico |
| F14 | structured 3-way JSON/YAML/TOML | ❌ | — |
| F15 | workbench 3-pane + fila | 🚧 | UI web pronta; backend de conflitos real ❌ |
| F16 | recipes + replay | 🚧 | `Store::add_recipe`; proptest de replay bit-idêntico ❌ |
| F17 | rules Starlark hermético | ❌ | `rules.rs` é regex/entropia, não Starlark |
| F18 | secrets regex+entropia | 🚧 | `rules::entropy` + scan |
| F19 | skills OWASP LLM | 🚧 | parcial em `rules::scan` |
| F20 | SBOM parsers lockfiles | 🚧 | `sbom::parse` (ecossistemas limitados) |
| F21 | OSV sync nightly | 🚧 | `osv::sync` esqueleto; delta ≤ 60 s não atingido |
| F22 | matcher semver/pep440 | ❌ | — |
| F23 | watch + auto-PR bump | ❌ | — |
| F24 | gate + SARIF + paridade CI | 🚧 | `gate::run_repo` + `sarif`; paridade local/CI ❌ |
| F25 | hook runner nativo | 🚧 | `doctor::install_hooks` básico |
| F26 | docs engine + doc cards | ❌ backend | UI pronta; geração O(diff) no daemon ❌ |
| F27 | FTS5 busca docs | ❌ | — |
| F28 | graph store edges | ❌ | grafo só existe no frontend (seed) |
| F29 | layout incremental + culling | 🚧 | culling no canvas da web; backend ❌ |
| F30 | anomaly EWMA/MAD | ❌ | UI com dados sintéticos; motor ❌ |
| F31 | cross-repo fanout/drift | ❌ | — |
| F32 | forge GitHub + rate budget | ❌ | — |
| F33 | GitLab + Forgejo | ❌ | — |
| F34 | rate budget por forge | ❌ | — |
| F35 | PRs agendados | ❌ | UI pronta; executor ❌ |
| F36 | auto-merge c/ lease | ❌ | — |
| F37 | webhooks in HMAC | ❌ | — |
| F38 | hooks out exec/HTTP/MQTT | ❌ | — |
| F39 | plugins Starlark packs | ❌ | — |
| F40 | plugins JSON-RPC stdio + caps | ❌ | — |
| F41 | WASM extism | ❌ | pós-MVP (só se houver demanda) |
| F42 | tiers T1/T2/T3 presets | 🚧 | `config.rs` + `colmeia.toml` (parcial) |
| F43 | companion T0 firmware | ❌ | — |
| F44 | temas claro/escuro/sépia/neumorph | ✅ | entregue (e expandido p/ 33 temas) |
| F45 | motion + responsivo web | ✅ | entregue (hambúrguer, breakpoints, ripple, sweep) |
| F46 | TUI paridade + grafo braille | ❌ | `tui.rs` é esqueleto (107 linhas) |
| F47 | onboarding wizard | 🚧 | `doctor::init_wizard` básico |
| F48 | doctor self-heal | 🚧 | `doctor::run` diagnostica; auto-cura ❌ |
| F49 | installers (cargo/brew/deb) | ❌ | — |
| F50 | docs + i18n PT/EN | ❌ | — |
| F51 | secrets age + keyring | ❌ | tokens em plaintext ❌ perigoso |
| F52 | redaction + fuzz parsers | ❌ | — |
| F53 | OTLP/LangSmith bridge | ❌ | — |
| F54 | tests/bench/chaos/proptest | 🚧 | 9 arquivos de teste + `repos_dir_test`; bench/ e chaos ❌ |
| F55 | governance RFC/DCO/licença | 🚧 | `rfcs/0001`; DCO/licença ❌ |

## Milestones (histórico Qwen)

| M | Estado | Lacuna principal |
|---|---|---|
| M0 Fundação | 🚧 | schema/config/GitBackend/CLI/doctor ok; farm generator + import 2.000 em ≤ 60 s + RSS idle ≤ 120 MB ❌ |
| M1 Sync | 🚧 | 3 syncs; leases; sweep ≤ 3 h e chaos ❌ |
| M2 TUI + Grafo | ❌ | TUI esqueleto; braille/virtualização ❌ |
| M3 Rules + Hooks | 🚧 | regex/entropia; Starlark + gate ≤ 10 s/500 arquivos ❌ |
| M4 Slice + Merge | 🚧 | plano ok; exec + drivers structured + proptest ❌ |
| M5 Segurança | 🚧 | SBOM/OSV parcial; matcher ❌ |
| M6 Docs + Anomalia | ❌ | engine de docs e EWMA/MAD ❌ |
| M7 Forges + CI | ❌ | GitHub adapter, PRs, webhooks ❌ |
| M8 Web + Plugins | 🚧 | web interativa ok (frontend); SSE real parcial; plugins ❌ |
| M9 Companion + Release | ❌ | firmware T0 + musl multi-arch ❌ |

**Linha de corte do MVP usável (M0–M4 + M7):** hoje ~35–40% — a UI web está à
frente do backend real; o caminho crítico é F12/F14/F16 (determinismo do slice
e merge), F30 e F32.

## Itens de maior risco / prioridade sugerida

1. **F12 + F16** — slice exec com CAS + recipes replayáveis (núcleo do valor).
2. **F51** — tokens em plaintext; age/keyring antes de qualquer beta.
3. **F21 + F22** — OSV sync real com matcher (segurança).
4. **F30** — anomalias EWMA/MAD (sinal cross-repo barato).
5. **F05 chaos + F54 bench** — contrato de confiabilidade/performance no CI.
6. **F10 path_sync** — dogfooding de skill-packs na frota.
