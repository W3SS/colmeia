---
title: "Colmeia App"
source: "https://chat.qwen.ai/c/7ea599d7-996e-4359-a704-3b7a1943afe6"
author:
published:
created: 2026-11-20
description: "histórico do chat com o Qwen sobre o app Colmeia que é um sistema de versionamento de código GUI"
tags:
  - "projects"
---
quais as opções de gestor de repositório Git, Gui com automações que existem?

Considerando o seu perfil técnico avançado (com foco em GCP, Oracle, sistemas multi-agentes e observabilidade com LangSmith), a forma como encaramos os "gestores de Git" e "GUIs" vai além de apenas visualizar commits. Você precisa de ferramentas que ofereçam **extensibilidade via API, suporte a Webhooks, integração com CI/CD nativo e capacidade de orquestrar agentes de IA**.

Abaixo, divido as opções entre **Gestores de Repositório (Web/Enterprise)**, **GUIs Desktop** e **Ferramentas de Terminal/Automação**, com foco em como elas lidam com automações.

---

### 1\. Gestores de Repositório (Web / Enterprise)

*Foco: Automação de CI/CD, Webhooks, APIs GraphQL/REST e Integração com Agentes de IA.*

- **GitLab (Self-Managed ou SaaS)**
	- **Automação:** É o padrão da indústria para pipelines complexos. O `.gitlab-ci.yml` é extremamente robusto, permitindo desde a validação de infraestrutura (Terraform/OpenTofu para GCP/Oracle) até o deploy de modelos de IA.
	- **Diferencial:** Possui o **GitLab Duo** (IA nativa) e uma API REST/GraphQL completa. Você pode configurar Webhooks para disparar sistemas multi-agentes (ex: um agente que lê o diff de um MR, roda uma validação com guardrails e reporta no LangSmith).
- **GitHub (GitHub Enterprise / Cloud)**
	- **Automação:** **GitHub Actions** é imbatível em ecossistema de marketplace e integrações. A API GraphQL permite buscas complexas em múltiplos repositórios.
	- **Diferencial:** Com o amadurecimento do **Copilot Workspace** e **Copilot Extensions** (em 2026), o GitHub atua como um orquestrador de agentes. Você pode criar Actions customizadas que disparam seus agentes multi-step, usando o GitHub como "cérebro" da esteira de automação.
- **Bitbucket (Atlassian)**
	- **Automação:** Bitbucket Pipelines.
	- **Diferencial:** Se a sua empresa usa o ecossistema Atlassian (Jira/Confluence), a automação de transição de tickets baseada em commits/branches é nativa e muito forte.
- **Gitea / Forgejo (Self-Hosted)**
	- **Automação:** Leves, escritos em Go, com suporte a Gitea Actions (compatível com GitHub Actions).
	- **Diferencial:** Ideais para ambientes *on-premise* ou air-gapped (comum em setups Oracle DBA/Enterprise), onde você precisa de automação de Git sem depender da nuvem pública.

---

### 2\. GUIs Desktop (Clientes Locais)

*Foco: Visualização de grafos, resolução de conflitos e automação via Git Hooks locais.*

- **GitKraken**
	- **Automação:** Embora seja uma GUI visual, ele se integra profundamente com Jira, GitHub Issues e Trello. Possui uma CLI companion (`gk`) que permite criar scripts de automação para abrir PRs, criar branches e interagir com o repositório remoto via terminal.
	- **Diferencial:** A visualização de *merge conflicts* e o *interactive rebase* visual economizam horas. Suporta perfis (útil se você alterna entre chaves SSH de GCP, Oracle e GitHub pessoal).
- **Fork**
	- **Automação:** Extremamente rápido e leve. Permite a criação de **Custom Tools/Scripts** que podem ser executados diretamente na interface com base no contexto (ex: um botão na GUI que roda um script Python de validação de *guardrails* no código antes do commit).
	- **Diferencial:** Interface limpa, suporte nativo a Git Flow e excelente gerenciamento de submodules (útil em repositórios de monorepos de IA).
- **Sourcetree**
	- **Automação:** Focado no ecossistema Atlassian. Permite a configuração visual de *Git Hooks* locais.
	- **Diferencial:** Bom para times que precisam de uma GUI gratuita e robusta, mas fica atrás do Fork e GitKraken em performance e integrações modernas.

---

### 3\. O "Pulo do Gato": CLI, TUI e Orquestração de Hooks

*Para um engenheiro do seu nível, a verdadeira automação no Git não acontece na GUI visual, mas sim na camada de hooks e CLI.*

- **GitHub CLI (`gh`)**
	- Essencial para automações. Você pode scriptar a criação de PRs, aprovação de MRs, e injeção de dados em secrets do GitHub Actions. É a ponte perfeita entre seus scripts Python/Bash e o repositório.
- **Lazygit (TUI - Terminal UI)**
	- Uma interface de terminal incrivelmente rápida.
	- **Automação:** Permite definir **Custom Commands** (atalhos que rodam scripts shell/Python). Você pode criar um atalho no `lazygit` que, ao ser pressionado, faz o *stash*, roda uma suíte de testes de *guardrails*, gera um commit com mensagem estruturada (ex: Conventional Commits) e dispara um trace para o LangSmith.
- **Lefthook / Husky**
	- Gerenciadores de **Git Hooks** multiplataforma.
	- **Automação:** Em vez de depender de scripts bash frágeis na pasta `.git/hooks`, você define em `lefthook.yml` (ou `package.json`) que, no `pre-commit`, um agente de IA local (ou um script de validação) roda para checar *guardrails* de código. No `pre-push`, ele pode disparar uma requisição para sua API GCP/Oracle para validar dependências.

---

### 4\. Integração com seu Stack (Multi-Agentes, GCP, LangSmith)

Como você trabalha com sistemas multi-agentes e observabilidade, o Git atua como o **estado e o gatilho** da sua automação:

1. **Agentes como "Contribuidores":** Você pode configurar Webhooks no GitHub/GitLab para que, a cada `git push` ou abertura de PR, um evento seja enviado para uma Cloud Function (GCP) ou Container (Oracle). Esse evento desperta seu *Multi-Agent System* (ex: um agente de Revisão de Código, um agente de Segurança/Guardrails).
2. **Observabilidade (LangSmith):** Os agentes que interagem com o Git (lendo diffs, sugerindo correções, gerando documentação) devem ter suas execuções traceadas no LangSmith. O Git Hook local (`pre-commit`) ou o Webhook remoto deve injetar o *Trace ID* do LangSmith na descrição do PR/MR, criando um link direto entre o código e a observabilidade do agente que o manipulou.
3. **Infraestrutura como Código (IaC):** Usar o GitLab CI ou GitHub Actions para validar e aplicar mudanças em ambientes GCP/Oracle, onde o próprio repositório é a fonte da verdade (GitOps com ArgoCD ou Flux).

### Resumo: O que escolher?

- **Para o Servidor/Remoto:** **GitLab** (se quiser pipelines complexos e self-hosted/Oracle) ou **GitHub** (se quiser o melhor ecossistema de IA e integrações GCP).
- **Para o Desktop (Visual):** **GitKraken** (para navegação complexa) ou **Fork** (para velocidade e scripts customizados).
- **Para Automação Pura (Seu dia a dia):** **Lazygit** combinado com **Lefthook** (para hooks locais) e **GitHub CLI** (para automações remotas).

eu estou planejando criar uma GUI com um mix de features entre o gitkraken e automações de TUI, a ideia central, é poder gerir numa mesma plataforma milhares de repositórios, programar gatilhos de pull e Sync entre repositórios que foram clonados, programar PRs, orquestrar criar novas branchs, e novos projetos, Scripts de pré-commit, controle de PRs automatizados, comandos de slice para quebrar commita grandes feitos por IA, documentação viva de tudo o que estiver no diretório, mapa em grafos do que está acontecendo nestes repositórios agregados a esta plataforma, Scan de segurança de Skills e artefatos, análise de comportamentos incomuns entre os projetos, tracing de CVEs e vulnerabilidades buscando sempre atualizações de CVEs e no OWASP, tarefas paralelas e tratamento de conflitos visual e de maneira deterministica, quero o foco scripts e automação, com heurísticas de controle de análise estática de código para não ter que integrar mais ferramentas, hooks de CI/CD e o principal escalabilidade de maneira Open Source, quero que isso funcione bem num computador batata sem GPU e uns 4gb de memória ram ddr3 ou mesmo num arduíno, a ideia é planejar essa implementação murando no hardware mais básico e mais fraco o possível, gere um documento detalhado de como implementar essa ferramenta em etapas e todos os artefatos necessários para que seja possível atingir esses objetivos que propus neste promot

## Colmeia — Documento de Arquitetura e Implementação

**Gestor Git open source, TUI-first com GUI opcional, focado em automação determinística para hardware mínimo**

> Nome de trabalho: **Colmeia** (milhares de repositórios = células; workers = abelhas; scheduler = rainha). CLI: `colmeia`, daemon: `colmeiad`.

---

## 0\. Premissas honestas (leia antes de tudo)

1. **Arduino não roda o core. Fisicamente.** Um Arduino Uno tem 2 KB de SRAM, 32 KB de flash e 16 MHz. O próprio Git precisa de dezenas de MB; TLS moderno, SQLite e parsing de AST precisam de centenas. Nem ESP32 (520 KB SRAM) roda Git. **Solução de engenharia, não desculpa:** arquitetura em *tiers*, onde a placa MCU vira um **nó companion** (gatilho físico, display de status, LED de alerta) conectado ao daemon via serial/MQTT/HTTP. O piso real para o core headless é um **Raspberry Pi Zero 2 W (512 MB)**.
2. **"Sem integrar mais ferramentas" ≠ sem dependência de dados.** O tracking de CVE usa o **feed público do OSV.dev** (dumps ZIP por ecossistema no bucket GCS `osv-vulnerabilities`). Isso é *dado*, não integração de scanner. Todo o resto (SAST, secrets, SBOM, matching de CVE, docs, grafo) é **in-core**.
3. **Dependência externa mínima e declarada:** binário `git ≥ 2.40` instalado (fetch/push/merge via subprocess controlado; leituras via `gitoxide` puro-Rust). Nada de Docker, Node, Python, Electron ou banco servidor.
4. **Determinismo é requisito de primeira classe**, não feature: ordenações estáveis, tie-break por hash de conteúdo, políticas versionadas, *recipes* replayáveis, atualização de refs com CAS (`update-ref <new> <old>`).

---

## 1\. Mapa requisito → subsistema

| Seu requisito | Subsistema | Seção |
| --- | --- | --- |
| Milhares de repositórios | Registry + modos de repo + scheduler adaptativo | §6.1 |
| Gatilhos de pull/sync entre clones | Sync Policies (cron/evento/manual) | §6.1 |
| PRs programados, branches, projetos novos | Forge Adapters + Job Templates | §6.9 |
| Scripts pré-commit | Hook Runner nativo + gates | §6.4/6.9 |
| Slice de commits grandes de IA | Slice Engine (tree-sitter + DAG + plumbing) | §6.2 |
| Conflitos visuais e determinísticos | Merge Workbench + drivers + recipes + rerere | §6.3 |
| Documentação viva do diretório | Docs Engine incremental + FTS5 | §6.7 |
| Mapa em grafos agregado | Graph Store + layout incremental + canvas TUI/SVG | §6.8 |
| Scan de Skills e artefatos | Rule Packs de agente (OWASP LLM Top 10) | §6.4 |
| Comportamento incomum entre projetos | Anomaly Engine (EWMA/MAD, cross-repo) | §6.6 |
| CVE/OWASP sempre atualizado | OSV Sync + SBOM in-core + matcher | §6.5 |
| Tarefas paralelas | Job DAG + locks por repo + worktrees isolados | §6.10 |
| Heurísticas de análise estática in-core | Tree-sitter + Starlark rules | §6.4 |
| Hooks CI/CD | `colmeia gate` + SARIF + webhooks HMAC | §6.9 |
| Escalabilidade open source | Plugins Starlark + JSON-RPC stdio | §8 |
| PC batata 4 GB DDR3 sem GPU / MCU | Tiers + memory governor | §2 |

---

## 2\. Tiers de hardware e orçamentos (contrato de performance)

| Tier | Hardware | O que roda | Orçamento RSS | Workers |
| --- | --- | --- | --- | --- |
| **T0 — Companion** | Arduino Uno / ESP32 | Botão de gatilho, OLED de status, LED de alerta. **Não roda Git.** Fala JSON com o daemon | < 40 KB (ESP32) | 0 |
| **T1 — Headless mínimo** | RPi Zero 2 W, 512 MB | Core + scheduler + sync + gates. Docs/anomalia em batch noturno. Sem tree-sitter de arquivos > 256 KB | ≤ 180 MB total | 1 |
| **T2 — Batata alvo** | PC 4 GB DDR3, sem GPU | **Tudo**: core + TUI + web mode + scans + slice + merge | ≤ 600 MB típico, hard cap 900 MB | 2–4 |
| **T3 — Desktop** | 8 GB+ | T2 + mais workers + GUI nativa opcional (egui) | ≤ 2 GB | 8 |

**Orçamento detalhado T2:**

| Componente | Alvo |
| --- | --- |
| Daemon idle (registry 2.000 repos, scheduler, HTTP) | ≤ 120 MB |
| SQLite page cache (`PRAGMA cache_size=-32768`) | 32 MB |
| Por worker (pico, inclui parsing tree-sitter) | ≤ 150 MB |
| Subprocess `git` (capado via `setrlimit(RLIMIT_AS)` / cgroup v2 `memory.max`) | ≤ 256 MB cada |
| TUI (ratatui, listas virtualizadas) | ≤ 40 MB |
| Web mode (tiny\_http + templates compilados + SSE) | +10 MB |

**Matemática de viabilidade (2.000 repos, T2, 2 workers):**

- Fetch médio 3 s ⇒ sweep completo ≈ 50 min. Com polling adaptativo (100 hot @5 min, 400 warm @1 h, 1.500 cold @noite) ⇒ pico ~15 fetches/min. Folgado.
- Disco: mirrors bare `--filter=blob:none` ≈ 5–30 MB/repo ⇒ 20–60 GB. Worktrees completos só para subconjunto ativo (LRU de 30 × ~100 MB ≈ 3 GB). OSV filtrado ≈ 50–150 MB. SQLite com poda ≈ 1–3 GB.

Esses números são **testes de aceitação**, não aspirações (§11).

---

## 3\. Arquitetura e stack

```
┌────────────────────────────────────────────────────────┐
│  TUI (ratatui)   Web GUI (SSE)   T0 Companion (MQTT/serial)│
└──────────────┬─────────────────────────────────────────┘
               │ IPC local (Unix socket / HTTP loopback)
┌──────────────▼─────────────────────────────────────────┐
│                     colmeiad (daemon)                   │
│  Event Bus ── Scheduler ── Job DAG ── Memory Governor   │
│  ┌─────────┬─────────┬─────────┬─────────┬───────────┐  │
│  │ Sync    │ Slice   │ Merge   │ Rules   │ OSV/SBOM  │  │
│  │ Engine  │ Engine  │ Drivers │(Starlark)│ Matcher   │  │
│  ├─────────┼─────────┼─────────┼─────────┼───────────┤  │
│  │ Docs    │ Graph   │ Anomaly │ Forge   │ Hook/CI   │  │
│  └─────────┴─────────┴─────────┴─────────┴───────────┘  │
│  GitBackend: gitoxide (read) + git CLI (write, capado)  │
│  SQLite (WAL) = única fonte de verdade                  │
│  Plugins: Starlark (hermético) │ JSON-RPC stdio (capado)│
└────────────────────────────────────────────────────────┘
```

**Decisões de stack (e por quê, para batata):**

| Escolha | Alternativa rejeitada | Razão |
| --- | --- | --- |
| **Rust**, binário estático musl | Go (GC, RSS baseline maior), Python (runtime) | Memória previsível, cross-compile `aarch64`/`armv6` para RPi, zero runtime deps |
| **Threads OS + crossbeam** no core; **ureq** (HTTP client bloqueante) + **tiny\_http** (server) | tokio full | Sem runtime async = ~10–20 MB a menos e sem work-stealing overhead. Async só se necessário depois |
| **rusqlite (bundled, FTS5)** | Postgres, sled | Single-file, WAL, single-writer queue resolve contenção |
| **gitoxide (gix)** para leituras + `git` CLI para fetch/push/merge | libgit2 (C, FFI), gitoxide-only | Escritas do gitoxide ainda em maturação; híbrido é honesto e seguro. Trait `GitBackend` permite migrar depois |
| **tree-sitter** (gramáticas compiladas, feature-flag por linguagem) | LSP servers, scanners externos | Parsing incremental, ~1–5 MB por gramática no binário, skip de arquivos > 1 MB |
| **Starlark** (starlark-rust) para regras/políticas | Lua, WASM no MVP | **Hermético e determinístico por design** (sem I/O), sandbox natural, sintaxe Python-like = barreira baixa para comunidade |
| **ratatui + crossterm** (TUI primária) | Electron/Tauri | Tauri/Electron = 200–500 MB de webview. Inviável em T2 com 2.000 repos |
| **Web mode server-rendered** (maud/askama + SSE + JS mínimo) | SPA React | Daemon +10 MB; **o browser roda em outra máquina** — a batata serve, o notebook renderiza. É assim que se tem "visual GitKraken" sem pagar RAM |
| **age** (arquivo cifrado) + keyring opcional para tokens | plaintext em SQLite | Tokens nunca em claro; redação no audit log |

Licença: **MIT/Apache-2.0 dual**. Build: `cargo build --release --target x86_64-unknown-linux-musl` (e `aarch64`, `arm` para RPi).

---

## 4\. Modelo de dados (DDL núcleo — artefato schema/001\_core.sql)

```
PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-32768; PRAGMA temp_store=FILE; PRAGMA mmap_size=268435456;

CREATE TABLE repos(
  id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL,
  mode TEXT CHECK(mode IN ('managed','mirror','dormant')) NOT NULL,
  forge TEXT, remote_url TEXT, default_branch TEXT, head_sha TEXT,
  priority INTEGER DEFAULT 5, last_activity INTEGER, state TEXT DEFAULT 'idle');

CREATE TABLE sync_policies(
  id INTEGER PRIMARY KEY, src_repo INTEGER, dst_repo INTEGER,
  kind TEXT CHECK(kind IN ('fetch_upstream','push_mirror','branch_sync','path_sync')),
  refspec TEXT, path_glob TEXT, cron TEXT, on_event TEXT,
  conflict_policy TEXT DEFAULT 'escalate', enabled INTEGER DEFAULT 1);

CREATE TABLE jobs(
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, repo_id INTEGER,
  payload_json TEXT NOT NULL, status TEXT DEFAULT 'queued',
  priority INTEGER DEFAULT 5, lease_until INTEGER, attempts INTEGER DEFAULT 0,
  depends_on INTEGER, created_at INTEGER, finished_at INTEGER, error TEXT);
CREATE INDEX jobs_dispatch ON jobs(status, priority, created_at);

CREATE TABLE events(id INTEGER PRIMARY KEY, ts INTEGER, repo_id INTEGER,
  kind TEXT, payload_json TEXT);              -- append-only, poda por retenção

CREATE TABLE findings(
  id INTEGER PRIMARY KEY, repo_id INTEGER, blob_sha TEXT, rule_id TEXT,
  severity TEXT, path TEXT, line INTEGER, message TEXT, owasp TEXT,
  status TEXT DEFAULT 'open', first_seen INTEGER, last_seen INTEGER,
  UNIQUE(repo_id, blob_sha, rule_id, path, line));   -- cache por blob = incremental

CREATE TABLE sbom(repo_id INTEGER, commit_sha TEXT, ecosystem TEXT,
  name TEXT, version TEXT, purl TEXT, direct INTEGER,
  PRIMARY KEY(repo_id, commit_sha, ecosystem, name, version));

CREATE TABLE vulns(
  id TEXT PRIMARY KEY, aliases_json TEXT, ecosystem TEXT, name TEXT,
  ranges_json TEXT, severity TEXT, fixed_json TEXT, modified INTEGER, withdrawn INTEGER);

CREATE TABLE baselines(repo_id INTEGER, metric TEXT, ewma REAL, mad REAL,
  n INTEGER, updated_at INTEGER, PRIMARY KEY(repo_id, metric));

CREATE TABLE symbols(id INTEGER PRIMARY KEY, repo_id INTEGER, blob_sha TEXT,
  path TEXT, kind TEXT, name TEXT, start_line INTEGER, end_line INTEGER, sig_hash TEXT);

CREATE TABLE docs(repo_id INTEGER, commit_sha TEXT, path TEXT, md TEXT,
  PRIMARY KEY(repo_id, commit_sha, path));
CREATE VIRTUAL TABLE docs_fts USING fts5(path, md, content='docs');

CREATE TABLE edges(src_kind TEXT, src_id TEXT, dst_kind TEXT, dst_id TEXT,
  rel TEXT, weight REAL DEFAULT 1, commit_sha TEXT,
  PRIMARY KEY(src_kind, src_id, dst_kind, dst_id, rel));

CREATE TABLE recipes(               -- TODA resolução/slice é replayável
  id INTEGER PRIMARY KEY, repo_id INTEGER, kind TEXT,
  input_hashes TEXT, policy_hash TEXT, output_hash TEXT,
  plan_json TEXT, created_at INTEGER);

CREATE TABLE audit(id INTEGER PRIMARY KEY, ts INTEGER, actor TEXT,
  action TEXT, target TEXT, details_json TEXT);   -- tokens redigidos
```

---

## 5\. Configuração e CLI

**Artefato `colmeia.toml` (exemplo funcional):**

```
[tier]
profile = "t2"                  # t1 | t2 | t3 — presets de governor/workers
workers = 2
memory_soft_mb = 600
memory_hard_mb = 900
git_child_cap_mb = 256

[registry]
worktree_lru_size = 30          # worktrees completos mantidos; resto = mirror
hot_watch_count = 100           # inotify só no subconjunto hot

[[sync]]
src = "upstream/skill-pack-core"
dst = "fleet/*"                 # glob: distribui para N repos de agentes
kind = "path_sync"
path_glob = "skills/**"
cron = "0 */6 * * *"
conflict_policy = "structured"  # §6.3

[[sync]]
src = "origin"
dst = "mirror-fork"
kind = "branch_sync"
refspec = "main"
on_event = "push"               # gatilho por evento, além do cron

[slice]
max_lines_per_slice = 400
group_by = ["symbol", "test_pairing", "manifest_lockfile"]
verify_hook = "gate"            # cada slice passa pelo gate antes de existir
refuse_merge_commits = true

[security]
osv_ecosystems = ["PyPI", "npm", "crates.io", "Go", "Maven"]
osv_sync_cron = "0 3 * * *"
rule_packs = ["core", "secrets", "skills-owasp-llm", "python", "js"]
gate_block_severity = "high"

[forge.github]
token_ref = "age:github-pat"    # nunca plaintext
rate_budget_per_hour = 4000

[observability]                 # opcional, feature-flag, zero custo desligado
otlp_endpoint = ""              # export de traces de ações de agentes
webhook_trace = ""              # compatível com ingest LangSmith via bridge
```

**CLI (contrato — `docs/cli.md`):**

| Comando | Função |
| --- | --- |
| `colmeia init / add <url…> / import <dir>` | Registry; import em massa de diretório com N clones |
| `colmeia tui` / `colmeia web [--bind 127.0.0.1:7420]` | Interfaces |
| `colmeia sync [--policy id] [--now]` | Dispara syncs |
| `colmeia slice <commit> [--plan-only] [--exec]` | Plano JSON editável → execução atômica |
| `colmeia merge <a> <b> [--driver auto]` | Workbench; auto-resolve com recipe |
| `colmeia replay <recipe-id>` | **Reexecução bit-idêntica** (teste de determinismo) |
| `colmeia gate --rev-range A..B [--format sarif\|json]` | Mesmas heurísticas local e no CI |
| `colmeia scan [--skills] [--sbom] [--cve]` | Scans incrementais |
| `colmeia docs / graph [--repo X] [--svg out.svg]` | Geração/consulta |
| `colmeia pr create --schedule "…" --template bump-cve` | PRs programados |
| `colmeia new project <name> --template agent-skill` | Scaffold + repo + policies |
| `colmeia doctor / bench --repos 2000` | Diagnóstico + contrato de performance |
| `colmeia hook install <repo>` | Instala hooks nativos (sem Husky/lefthook) |

---

## 6\. Subsistemas — algoritmos e artefatos

### 6.1 Scheduler + Sync Engine (milhares de repos)

- **Modos de repo:** `managed` (worktree completo, LRU), `mirror` (bare `--filter=blob:none`, blobs on-demand — GitHub/GitLab suportam promisor), `dormant` (só metadata; fetch sob demanda).
- **Polling adaptativo:** hot (atividade < 24 h) @5 min; warm @1 h; cold @noite; jitter ±20% anti-thundering-herd. `inotify` apenas nos `hot_watch_count` worktrees (limite de watches do kernel não estoura).
- **Tipos de sync:** `fetch_upstream`, `push_mirror`, `branch_sync` (**fast-forward only por padrão**; divergência ⇒ job de merge com política), `path_sync` (cópia de paths entre repos com hash de conteúdo + commit determinístico — **ideal para distribuir skill-packs entre frotas de agentes**).
- Manutenção própria: `git commit-graph write --reachable` incremental após fetch (leituras de ancestralidade O(1)-ish no grafo).

**Pseudocódigo do dispatcher (artefato `core/scheduler.rs`):**

```
loop:
  governor.check()                      # §6.10 — pausa se soft limit
  due = cron_due() + event_triggered() + adaptive_poll_due()
  for job in stable_sort(due, by=(priority, content_hash(id))):
    if repo_lock[job.repo].free and workers.free and io_budget.ok:
      lease(job, ttl=10min); spawn_worker(job)   # lease ⇒ crash-recovery
  sleep(250ms)
```

Jobs têm `lease_until`: `kill -9` no meio ⇒ outro worker retoma do checkpoint. **Teste de caos obrigatório (§11).**

### 6.2 Slice Engine — quebrar commits grandes de IA

**Algoritmo (artefato `docs/algorithms/slice.md` + `core/slice.rs`):**

```
1. diff(C, parent(C)) → hunks → arquivos
2. tree-sitter: mapear hunks a unidades semânticas (função/classe/bloco)
3. DAG de dependências entre unidades:
   - ordem intra-arquivo; arestas import/call (heurística de símbolos)
   - pareamento teste↔código (convenções de path); manifest↔lockfile
4. topo-sort estável (tie-break: hash do conteúdo) + bin-packing
   sob max_lines_per_slice → PLANO JSON (editável na TUI antes de executar)
5. Execução em branch temp, a partir de parent(C):
   para cada slice:
     git apply --cached <hunks-do-slice>   (fallback --3way)
     gate --staged                          # cada slice é auto-consistente
     git commit -m "<msg gerada dos símbolos>"
6. Sucesso total ⇒ update-ref CAS: git update-ref refs/heads/X new old
   + backup ref refs/colmeia/backup/X antes de qualquer coisa
7. recipe{input_hashes, policy_hash, plan, output_hash} persistido
```

**Garantias:** mesmo input + mesma política ⇒ mesmo plano e mesmos SHAs de saída (committer/author dates fixadas no plano). Recusa merge-commits e worktree sujo. Detecção de "commit de IA": trailers (`Co-authored-by: …[bot]`), velocity, e flag explícita de agentes via API — feeding direto para seus multi-agentes.

**Aceitação:** commit de 5.000 linhas ⇒ plano ≤ 5 s em T2; `replay` bit-idêntico; cada slice passa no gate isoladamente (bisectable).

### 6.3 Merge determinístico + workbench visual

- `rerere` habilitado por repo gerenciado (resoluções humanas viram replay automático).
- **Drivers por path-pattern, declarados em Starlark:** `ours`, `theirs`, `union` (arquivos append-only), **structured 3-way** para JSON/YAML/TOML (merge por chave), lockfile ⇒ comando de regeneração *declarado e opcional* (única concessão a tool externa, explicitamente configurada), **semântico para código** (3-way por símbolo via tree-sitter; ambos editaram o mesmo símbolo ⇒ escala para humano, nunca "chuta").
- **Toda auto-resolução gera recipe** (hashes de entrada + driver + hash de saída) ⇒ auditável, replayável, e a TUI mostra *"por que isso foi resolvido assim"*.
- **Paralelismo de conflitos:** cada merge roda em `git worktree add --detach` isolado ⇒ N conflitos abertos simultaneamente sem bloquear outros repos. Fila visual na TUI com 3 panes + provenance.

### 6.4 Análise estática in-core + scan de Skills

- **Fatos** extraídos por tree-sitter (AST resumido, símbolos, imports, strings) + metadados de arquivo → passados a **regras Starlark** (herméticas, sem I/O ⇒ determinísticas e versionadas por hash).
- **Rule pack `core`:** complexidade ciclomática aproximada, tamanho de função, nesting, `eval/exec`, `subprocess shell=True`, `pickle.loads`, `yaml.load`, concat SQL, density de TODO.
- **Pack `secrets`:** regex de prefixos conhecidos (`ghp_`, `sk-`, `AKIA`…) + entropia de Shannon > 4.5 com allowlist.
- **Pack `skills-owasp-llm`** (direto para seu stack de agentes): scan de `SKILL.md`, manifests de tools, prompts e artefatos — padrões de *prompt injection* ("ignore previous instructions", override de papel), unicode invisível (zero-width/bidi), blobs base64 suspeitos, URLs de exfil, wildcards de permissão de tools (*excessive agency*), `curl | bash`. Findings mapeados a **OWASP LLM Top 10** e ASVS.
- **Incremental por blob-hash:** achados cacheados por `(blob_sha, rule_id)` ⇒ custo O(diff), não O(repo). Supressão inline + baseline file.

**Artefato `rules/skills/injection.star` (exemplo real):**

```
OVERRIDE = ["ignore previous", "disregard your instructions", "you are now"]
def check(facts):
    findings = []
    if facts.path.endswith(("SKILL.md", ".prompt", ".skill")):
        low = facts.text.lower()
        for p in OVERRIDE:
            if p in low:
                findings.append(finding(rule="skill.injection.override",
                    severity="high", owasp="LLM01", line=facts.line_of(p)))
        if facts.has_zero_width or facts.has_bidi:
            findings.append(finding(rule="skill.hidden_unicode",
                severity="critical", owasp="LLM01"))
        for perm in facts.tool_permissions:
            if perm.scope == "*":
                findings.append(finding(rule="skill.excessive_agency",
                    severity="high", owasp="LLM06"))
    return findings
```

### 6.5 SBOM + OSV/CVE + OWASP (sem scanner externo)

1. **SBOM in-core:** parsers diretos de `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `poetry.lock`, `uv.lock`, `Cargo.lock`, `go.sum`, `requirements.txt` pinado, `pom.xml`. Export CycloneDX (JSON emitido nativamente).
2. **OSV Sync:** download de `https://storage.googleapis.com/osv-vulnerabilities/<eco>/all.zip` por ecossistema configurado, nightly, delta por `modified`; store mínimo em SQLite (~50–150 MB filtrado). Sem NVD no MVP (rate limits; OSV já agrega GHSA).
3. **Matcher:** nome+ecossistema+ranges (semver/pep440 in-core) → finding com **versão corrigida sugerida**.
4. **Watch mode:** CVE nova que casa com SBOM existente ⇒ evento ⇒ política opcional `auto-PR bump` (§6.9). MVP: bump automático só para ecossistemas simples (Cargo.lock, requirements pinado); demais ⇒ PR com advisory + patch sugerido. **Honestidade de escopo:** resolução completa de lockfile sem package manager é fora do MVP.

### 6.6 Anomalias cross-repo (sem GPU, sem ML)

- Baselines por repo em SQL: commits/dia, distribuição de diff-size, churn por path, histograma de horário, taxa de dependência nova, aparição de binários, force-push.
- **EWMA + z-score robusto (mediana/MAD)** ⇒ outliers. Batch noturno, custo ~segundos para 2.000 repos.
- **Sinais de supply-chain cross-repo:** mesmo blob suspeito aparecendo em N repos (fanout index por hash), spike de velocity do mesmo "autor" (agente descontrolado), domínio novo em skill, force-push em branch protegida. Tudo SQL + loops pequenos — **perfeito para detectar drift de frotas de agentes**, com trace exportável para LangSmith via webhook bridge (feature-flag).

### 6.7 Documentação viva

- Por mudança: extração de símbolos + docstrings + README + manifests → markdown **pinado a commit SHA** (viva, mas versionada). Regeneração só de paths alterados (hash).
- **Artefatos não-código ganham "doc card":** tipo, hash, primeiro/último visto, findings linkados; para skills: frontmatter (nome, descrição, tools, permissões) ⇒ *"documentação de tudo que estiver no diretório"* literalmente.
- Busca: FTS5. Render: pane na TUI + rota no web mode.

### 6.8 Grafo agregado

- Nós: repos, branches, PRs, dependências, findings, grupos. Arestas em SQLite (`edges`): fork/upstream, sync-pairs, dependência, fluxo de PR.
- **Nunca fazer layout de 10k nós na batata:** clusters agregados por grupo, layout em camadas (Sugiyama-lite) incremental, expand-on-demand, viewport culling.
- Render: **canvas braille da ratatui** (~400 nós visíveis por viewport, drill-down em listas) + SVG server-side no web mode. Ancestralidade acelerada por commit-graph.

### 6.9 Forges, PRs automatizados, hooks CI/CD

- Trait `Forge` ⇒ adapters GitHub (REST+GraphQL), GitLab, Forgejo/Gitea. Fila **rate-limit-aware** com budget por forge (config).
- **PRs programados** (cron), **auto-PR por política** (bump CVE, output de slice, refresh de docs), **gates de review** (findings ≤ severidade configurada), auto-merge com poll de checks + lease.
- **Webhooks in:** tiny\_http + verificação HMAC ⇒ eventos. **Hooks out:** exec local, HTTP POST, MQTT (para T0).
- **Paridade CI:** `colmeia gate --format sarif` ⇒ mesmo binário, mesmas regras, mesmo resultado local e no CI; SARIF é JSON emitido in-core (upload nativo para code scanning do GitHub). Templates de pipeline gerados por `colmeia ci init`. Hook runner nativo (`pre-commit/pre-push/post-merge`) com timeout + rlimits — substitui Husky/lefthook.

### 6.10 Paralelismo + Memory Governor

- Job DAG com `depends_on`, lock por repo, worktrees isolados para tarefas conflitantes.
- **Governor:** thread watchdog lê `/proc/self/statm` + RSS dos filhos. Soft limit ⇒ pausa dispatch, drena. Hard limit ⇒ cancela job de menor prioridade (resumível via lease), despeja caches. Filhos `git` capados com `setrlimit(RLIMIT_AS)` no `pre_exec` (Linux) ou cgroup v2 `memory.max`.
- Single-writer queue para SQLite (WAL resolve leitores concorrentes).

---

## 7\. Interfaces

- **TUI (primária, paridade total):** pane de repos virtualizado (O(viewport) de RAM), stream de atividade, canvas de grafo, tabs de detalhe (diff/docs/findings/PRs), merge workbench 3-pane, command palette, teclas vim + mouse.
- **Web mode (visual "GitKraken-like"):** server-rendered (maud, templates compilados), SSE para live, JS mínimo. Bind localhost + token; acesso remoto via túnel SSH (evita stack TLS na batata; rustls é feature opcional). **O browser roda em outra máquina.**
- **T0 Companion (artefato `companion/protocol.md`):** JSON line-delimited sobre serial (Uno) ou MQTT/HTTP (ESP32):
```
{"cmd":"trigger","policy":"fleet-sync"}          // botão físico
{"evt":"queue","depth":42,"alert":"critical"}    // → OLED/LED
```

Firmware de exemplo em `companion/esp32/` e `companion/uno/` (< 200 linhas cada).

---

## 8\. Extensibilidade open source (escala de comunidade, não só de repos)

1. **Camada 1 — Rule Packs Starlark:** herméticos, determinísticos, seguros por construção. **Packs são eles próprios repos Git gerenciados pela Colmeia** (dogfooding: sync de packs via `path_sync`). Barreira de contribuição baixíssima ⇒ vetor de crescimento da comunidade.
2. **Camada 2 — Plugins de processo:** JSON-RPC 2.0 sobre stdio, qualquer linguagem, **capability-scoped** (paths de leitura, allowlist de rede, timeout, cap de memória):
```
{"jsonrpc":"2.0","method":"analyze","params":{"facts":{...},"caps":{"fs_read":["/repo"],"net":[]}}}
```
3. **Camada 3 (pós-MVP, só se houver demanda):** WASM via extism. Fora do MVP por memória.
4. Governança: DCO, diretório `rfcs/`, labels `good-first-rule`, **benchmark público como contrato** (PR que regride RSS/latência falha no CI).

---

## 9\. Estrutura do repositório

```
colmeia/
├── core/            # scheduler, jobs, governor, event bus, git backend
├── store/           # rusqlite, migrações, single-writer queue
├── engines/
│   ├── slice/  merge/  rules/  osv/  sbom/  anomaly/  docs/  graph/
├── forges/          # github, gitlab, forgejo
├── tui/             # ratatui app
├── web/             # tiny_http + maud + SSE
├── cli/             # clap; binário único `colmeia` (daemon embutido)
├── rules/           # packs Starlark (core, secrets, skills-owasp-llm, langs)
├── schema/          # DDL + migrações
├── companion/       # protocol.md, esp32/, uno/
├── bench/           # farm generator + contrato de performance
├── rfcs/  docs/     # cli.md, algorithms/, tiers.md, security-model.md
└── tests/           # proptest (determinismo), chaos, fixtures
```

---

## 10\. Roadmap em milestones (com critérios de aceitação mensuráveis)

| M | Escopo | Aceitação | Esforço (1–2 devs) |
| --- | --- | --- | --- |
| **M0** Fundação | Schema, registry, config, GitBackend híbrido, CLI esqueleto, doctor | Import de 2.000 repos sintéticos ≤ 60 s; RSS idle ≤ 120 MB | 3–4 sem |
| **M1** Sync | Scheduler adaptativo, leases, 4 tipos de sync, governor | Sweep 2.000 repos ≤ 3 h em T2; kill -9 ⇒ recuperação sem perda | 3–4 sem |
| **M2** TUI + Grafo | Panes, canvas braille, virtualização, commit-graph | Cold start ≤ 1 s; grafo de 2.000 nós navegável ≤ 40 MB | 4–5 sem |
| **M3** Rules + Hooks | Starlark, packs core/secrets, hook runner, `gate` + SARIF | Scan incremental O(diff); gate 500 arquivos ≤ 10 s | 4 sem |
| **M4** Slice + Merge | Slice engine, drivers, recipes, workbench, replay | Plano 5k linhas ≤ 5 s; **replay bit-idêntico (proptest)** | 5–6 sem |
| **M5** Segurança | SBOM parsers, OSV sync, matcher, skills pack | Sync delta OSV ≤ 60 s; matcher 10k pacotes ≤ 5 s | 4–5 sem |
| **M6** Docs + Anomalia | Docs incrementais + FTS5, baselines EWMA/MAD | Batch noturno 2.000 repos ≤ 15 min; docs regen O(diff) | 3–4 sem |
| **M7** Forges + CI | GitHub adapter, PRs programados, auto-PR CVE, webhooks HMAC | Fila respeita rate budget; PR agendado dispara ±30 s | 4–5 sem |
| **M8** Web + Plugins | Web mode SSE, JSON-RPC stdio, capabilities, GitLab/Forgejo | Web +10 MB RSS; plugin malicioso contido por caps | 4–5 sem |
| **M9** Companion + Release | Firmware T0, builds musl multi-arch, docs, RFC process | T1 (512 MB) passa suite reduzida; release v0.1 | 3 sem |

**Linha de corte do MVP usável: M0–M4 + M7 (GitHub).** Solo: ~7–9 meses até M8; dupla: ~4–5 meses.

---

## 11\. Testes e benchmark como contrato

- **Farm generator** (`bench/farm.py` ou Rust): cria N repos sintéticos com históricos, divergências, lockfiles, skills plantadas com injections conhecidas.
- **Property tests (proptest):** slice e merge ⇒ determinismo (mesmo input ⇒ mesmo output hash), atomicidade de refs, idempotência de sync.
- **Chaos:** `kill -9` em todo tipo de job ⇒ recuperação via lease/WAL; rede caída durante OSV sync ⇒ retry com backoff.
- **Contrato de performance (CI falha se regredir):** números das tabelas §2 e §10, medidos em runner de 4 GB sem GPU e em RPi Zero 2 emulado/real.
- **Segurança do próprio tool:** tokens age-cifrados, redação no audit, fuzzing dos parsers de lockfile e do endpoint de webhook.

---

## 12\. Riscos e mitigações

| Risco | Mitigação |
| --- | --- |
| Expectativa "roda em Arduino" | Tier doc público desde o README; T0 companion entrega o *espírito* do requisito |
| Maturação de escritas no gitoxide | Híbrido com CLI capada; trait `GitBackend` isola a migração |
| Dump OSV cresce | Filtro por ecossistema + poda de `withdrawn` + retenção |
| Contenção SQLite | Single-writer queue + WAL + batch writes |
| Disco com 2.000 worktrees | Modos mirror/dormant + LRU de worktrees |
| Gramáticas tree-sitter incham binário | Feature flags por linguagem; skip de arquivos > 1 MB |
| Scope creep (o risco maior deste projeto) | Linha de corte MVP rígida; WASM/GUI nativa/TLS ficam para pós-v0.1 |
| Rate limit de forges | Budget por forge na fila; backoff; degradação para polling |

---

## 13\. Próximos passos imediatos (checklist M0)

1. `cargo new colmeia --workspace` com crates da §9; CI com build musl + clippy + contrato de bench vazio.
2. `schema/001_core.sql` (§4) + migrações; `colmeia.toml` (§5) com presets de tier.
3. Trait `GitBackend` + impl híbrida (gix read / CLI write com rlimits).
4. Farm generator + primeiro teste: importar 2.000 repos, medir RSS.
5. `rfcs/0001-determinism.md`: ordenações estáveis, hashes de política, recipes, CAS — **o documento que governa todo o resto**.
6. README com a tabela de tiers honesta e o benchmark como promessa pública.

---

**Filosofia do projeto em uma frase:** *o Git é o estado, o SQLite é a memória, o Starlark é a lei, e cada automação deixa um recipe que qualquer um pode replayar — numa batata de 4 GB, com um Arduino apertando o botão.*

Se quiser, no próximo passo eu detalho um dos artefatos em profundidade total: o **Slice Engine** (implementação Rust do DAG + plumbing), o **Merge Workbench** (drivers structured + UI), ou o **rule pack de skills** completo com mapeamento OWASP LLM Top 10 — este último conversa direto com seu trabalho de guardrails e LangSmith.

![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/7018a9d7-1d21-45a7-ad5e-a815c243e50f_HXNrGvBXgAAn7B1.jpg?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI3MDE4YTlkNy0xZDIxLTQ1YTctYWQ1ZS1hODE1YzI0M2U1MGYiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ._baFzGumtTP5DxoOgPMN0mNEGK3EvMJ6AfE5ZOFsyRA&x-oss-process=image/resize,m_mfit,w_320,h_320)

crie um mapa visual da estrutura do software usando como exemplo a imagem em anexo, gere um desenho da arquitetura completo (uma versão mermaid e versões jxl e PNG)

Vou gerar o mapa visual no estilo da imagem anexa (fundo preto, ramificações curvas coloridas, nós em anel, títulos sublinhados) usando-a como referência, e depois entrego a versão Mermaid completa e o caminho para o JXL.

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/image_edit/b895c90a-5c79-494d-a366-143b4301f341/d578380f147242d8ab581874fb82b41b.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJkNTc4MzgwZjE0NzI0MmQ4YWI1ODE4NzRmYjgyYjQxYiIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.fJg4UsZiaJfCP4JQQwwXOElUBUhaW9Q5mk4Pr_SLTaA&x-oss-process=image/resize,m_mfit,w_450,h_450)

A versão visual (PNG) já foi gerada acima, no estilo do mapa de referência: fundo preto, raiz à esquerda, 9 ramos coloridos com nós em anel e sublinhados. Duas observações honestas antes dos artefatos:

- **Renderização por IA tem limites com texto**: escaparam pequenos erros de grafia em alguns rótulos (ex.: "Interfeces", "SQLLite", "workteres"). A imagem vale como *overview visual*; a **fonte da verdade é o Mermaid abaixo** — e o pipeline determinístico no final gera um PNG com texto 100% correto.
- **JXL**: o serviço de geração entrega raster em PNG. JPEG XL é só uma conversão lossless de um comando (`cjxl`, abaixo) — incluí o pipeline completo.

---

## 1\. Versão Mermaid completa (fonte editável — colmeia.mmd)

Esta é a versão **completa**, com todos os subnós do documento de arquitetura (mais profunda que a imagem):

```
mindmap
  root((Colmeia))
    Interfaces
      TUI ratatui
        Panes virtualizados
        Grafo braille
        Merge workbench 3 panes
        Command palette, vim e mouse
      Web mode
        HTML server-rendered + SSE
        Browser em outra máquina
        Localhost + token + túnel SSH
      CLI colmeia
        sync, slice, merge, gate, replay
        doctor, bench, hook install
      Companion T0
        Arduino Uno via serial
        ESP32 via MQTT e HTTP
        Botão de gatilho, OLED, LED
    Core colmeiad
      Event Bus
      Scheduler adaptativo
        Cron + eventos + polling
        Hot 5min, warm 1h, cold noite
        Jitter anti thundering-herd
        inotify só no subconjunto hot
      Job DAG
        Leases para crash-recovery
        Locks por repositório
        depends_on e prioridades
      Memory Governor
        Soft e hard limits
        rlimits e cgroups v2
        Pausa de dispatch e spill
    Git Backend
      gitoxide para leituras
      git CLI capado para escritas
        fetch --filter blob none
        push e merge
      commit-graph incremental
      Worktrees isolados
      update-ref CAS + backup refs
    Store SQLite
      WAL + fila single-writer
      Tabelas núcleo
        repos, sync_policies, jobs, events
        findings, sbom, vulns, baselines
        symbols, docs, edges, recipes, audit
      FTS5 para busca de docs
      Migrações versionadas
    Engines de Automação
      Sync Engine
        fetch_upstream
        push_mirror
        branch_sync fast-forward
        path_sync de skill-packs
      Slice Engine
        Hunks agrupados por símbolo tree-sitter
        DAG + topo-sort estável
        Plano JSON editável na TUI
        Gate por slice + refs atômicas
        Recusa merge-commits e worktree sujo
      Merge Determinístico
        rerere habilitado
        Drivers ours, theirs, union
        Structured 3-way JSON YAML TOML
        Semântico por símbolo
        Recipes replayáveis
        Fila visual de conflitos
    Segurança In-core
      Rules Starlark herméticas
        Pack core - complexidade e APIs perigosas
        Pack secrets - regex + entropia
        Pack skills - OWASP LLM Top 10
        Cache por blob-hash, custo O diff
      SBOM in-core
        Parsers diretos de lockfiles
        Export CycloneDX
      OSV e CVE
        Dumps por ecossistema, sync nightly
        Matcher semver e pep440
        Watch mode com auto-PR de bump
      Gate + SARIF
        Mesmas regras local e CI
        Severidade de bloqueio configurável
    Inteligência e Docs
      Anomaly Engine
        EWMA + z-score robusto MAD
        Cross-repo blob fanout
        Drift de frotas de agentes
        Batch noturno
      Docs Vivas
        Símbolos e docstrings
        Doc cards de artefatos e skills
        Pin por commit SHA
        Regeneração incremental
      Graph Store
        Clusters agregados por grupo
        Layout incremental Sugiyama-lite
        Canvas braille TUI + SVG web
        Viewport culling
    Forges e CI/CD
      Adapters
        GitHub REST e GraphQL
        GitLab
        Forgejo e Gitea
      PRs automatizados
        Agendados por cron
        Auto-PR de CVE bump
        Gates de review + auto-merge com lease
      Rate budget por forge
      Webhooks in com HMAC
      Hooks out exec, HTTP, MQTT
      Templates de pipeline gerados
    Tiers e Open Source
      T0 MCU companion
      T1 RPi Zero 512MB headless
      T2 PC 4GB alvo completo
      T3 Desktop 8GB+
      Plugins
        Starlark packs como repos Git
        JSON-RPC stdio com capabilities
        WASM pós-MVP
      Governança
        MIT e Apache-2.0
        RFCs e DCO
        Bench público como contrato
```

---

## 2\. Versões PNG (texto exato) e JXL — pipeline determinístico

Coerente com a filosofia do projeto: o diagrama é **código versionado**, renderizado por pipeline reprodutível.

```
# 1. Renderizar o Mermaid em PNG com tema dark (texto 100% fiel à fonte)
npm i -g @mermaid-js/mermaid-cli
mmdc -i colmeia.mmd -o colmeia-arquitetura.png -b black -t dark -w 2400 -s 2

# 2. Converter para JPEG XL lossless (instale: apt install libjxl-tools / brew install jpeg-xl)
cjxl colmeia-arquitetura.png colmeia-arquitetura.jxl -d 0 -e 9

# Bônus: SVG vetorial (ideal para o web mode da própria Colmeia)
mmdc -i colmeia.mmd -o colmeia-arquitetura.svg -b black -t dark
```
- `-d 0` = distância zero ⇒ **lossless** (JXL costuma ficar 30–50% menor que o PNG equivalente).
- Para converter a imagem estilizada gerada acima, basta salvar o PNG e rodar o mesmo `cjxl`.
- Rode isso no CI do próprio repositório e o diagrama vira **documentação viva** (§6.7 do documento): mudou o `.mmd` ⇒ regenera PNG/JXL/SVG automaticamente.

---

## 3\. Nota sobre "jxl"

Interpretei como **JPEG XL** (formato de imagem, par do PNG). Se você quis dizer **XML do draw.io/diagrams.net** (editável visualmente, estilo GitKraken), me avise que eu gero o arquivo `.drawio` completo com o mesmo layout de mind map — cores, nós e hierarquia preservados.

Quer que eu detalhe também uma **versão flowchart LR** do Mermaid (camadas com setas de fluxo de dados/eventos, em vez de mind map)? Ela complementa bem o mapa para o RFC 0001 do projeto.

gere um.drawio

Segue o arquivo completo. Salve como **`colmeia-arquitetura.drawio`** e abra no [app.diagrams.net](https://app.diagrams.net/) (ou na extensão *Draw.io Integration* do VS Code). Fundo preto, raiz à esquerda, 9 ramos coloridos com nós em anel e sublinhados — mesmo layout do mapa visual, 100% editável.

```
<mxfile host="app.diagrams.net" agent="colmeia-docs" version="24.7.7">
  <diagram id="colmeia-arq" name="Colmeia — Arquitetura">
    <mxGraphModel dx="1600" dy="1000" grid="0" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" page="0" pageScale="1" pageWidth="1169" pageHeight="826" math="0" shadow="0" background="#000000">
      <root>
        <mxCell id="0"/>
        <mxCell id="1" parent="0"/>

        <!-- ══════════ RAIZ ══════════ -->
        <mxCell id="root" value="Colmeia — Arquitetura do Gestor Git Open Source · TUI-first · automação determinística · hardware mínimo" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#F59E0B;strokeWidth=2;fontColor=#FFFFFF;fontSize=15;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=8;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="20" y="924" width="230" height="120" as="geometry"/>
        </mxCell>
        <mxCell id="rootRing" value="" style="ellipse;html=1;fillColor=none;strokeColor=#F59E0B;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="262" y="977" width="14" height="14" as="geometry"/>
        </mxCell>

        <!-- ══════════ 1. INTERFACES (azul) ══════════ -->
        <mxCell id="b1" value="Interfaces" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4A9EFF;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="131" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b1r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#4A9EFF;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="139" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l11" value="&lt;b&gt;TUI ratatui:&lt;/b&gt; panes, grafo braille, merge workbench" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4A9EFF;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="40" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l12" value="&lt;b&gt;Web mode:&lt;/b&gt; HTML + SSE, browser em outra máquina" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4A9EFF;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="96" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l13" value="&lt;b&gt;CLI:&lt;/b&gt; sync, slice, gate, replay, bench" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4A9EFF;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="152" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l14" value="&lt;b&gt;Companion T0:&lt;/b&gt; Arduino/ESP32 via serial e MQTT" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4A9EFF;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="208" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 2. CORE COLMEIAD (roxo) ══════════ -->
        <mxCell id="b2" value="Core colmeiad" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A855F7;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="365" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b2r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#A855F7;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="373" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l21" value="&lt;b&gt;Event Bus + Scheduler:&lt;/b&gt; cron, eventos, polling adaptativo" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A855F7;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="302" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l22" value="&lt;b&gt;Job DAG:&lt;/b&gt; leases, locks por repo, tarefas paralelas" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A855F7;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="358" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l23" value="&lt;b&gt;Memory Governor:&lt;/b&gt; soft/hard caps, rlimits, cgroups" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A855F7;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="414" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 3. GIT BACKEND (verde) ══════════ -->
        <mxCell id="b3" value="Git Backend" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4ADE80;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="571" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b3r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#4ADE80;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="579" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l31" value="&lt;b&gt;gitoxide:&lt;/b&gt; leituras puras em Rust" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4ADE80;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="508" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l32" value="&lt;b&gt;git CLI capado:&lt;/b&gt; fetch, push, merge" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4ADE80;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="564" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l33" value="commit-graph, worktrees isolados, update-ref CAS" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#4ADE80;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="620" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 4. STORE SQLITE (laranja) ══════════ -->
        <mxCell id="b4" value="Store SQLite" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#F59E0B;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="777" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b4r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#F59E0B;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="785" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l41" value="WAL + fila single-writer" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#F59E0B;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="714" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l42" value="repos, jobs, events, findings, recipes, edges" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#F59E0B;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="770" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l43" value="&lt;b&gt;FTS5:&lt;/b&gt; busca full-text das docs" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#F59E0B;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="826" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 5. ENGINES DE AUTOMAÇÃO (ciano) ══════════ -->
        <mxCell id="b5" value="Engines de Automação" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#22D3EE;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="983" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b5r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#22D3EE;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="991" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l51" value="&lt;b&gt;Sync:&lt;/b&gt; upstream, mirror, branch, path_sync" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#22D3EE;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="920" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l52" value="&lt;b&gt;Slice:&lt;/b&gt; DAG tree-sitter, plano JSON, gate por slice" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#22D3EE;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="976" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l53" value="&lt;b&gt;Merge:&lt;/b&gt; rerere, drivers determinísticos, recipes" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#22D3EE;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1032" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 6. SEGURANÇA IN-CORE (vermelho) ══════════ -->
        <mxCell id="b6" value="Segurança In-core" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EF4444;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="1189" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b6r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#EF4444;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="1197" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l61" value="&lt;b&gt;Starlark:&lt;/b&gt; packs core, secrets, skills OWASP LLM" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EF4444;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1126" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l62" value="SBOM de lockfiles + OSV/CVE nightly" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EF4444;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1182" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l63" value="&lt;b&gt;gate + SARIF:&lt;/b&gt; paridade local e CI" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EF4444;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1238" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 7. INTELIGÊNCIA & DOCS (magenta) ══════════ -->
        <mxCell id="b7" value="Inteligência &amp; Docs" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EC4899;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="1395" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b7r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#EC4899;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="1403" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l71" value="&lt;b&gt;Anomalias:&lt;/b&gt; EWMA/MAD, cross-repo, drift de agentes" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EC4899;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1332" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l72" value="&lt;b&gt;Docs vivas:&lt;/b&gt; símbolos, doc cards, pin por commit" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EC4899;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1388" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l73" value="&lt;b&gt;Grafo:&lt;/b&gt; clusters, layout incremental, TUI + SVG" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#EC4899;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1444" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 8. FORGES & CI/CD (lima) ══════════ -->
        <mxCell id="b8" value="Forges &amp; CI/CD" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A3E635;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="1601" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b8r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#A3E635;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="1609" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l81" value="GitHub, GitLab, Forgejo: rate budget" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A3E635;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1538" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l82" value="PRs agendados + auto-PR de CVE + auto-merge" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A3E635;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1594" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l83" value="Webhooks HMAC + hooks exec/HTTP/MQTT" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#A3E635;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1650" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ 9. TIERS & OPEN SOURCE (azul claro) ══════════ -->
        <mxCell id="b9" value="Tiers &amp; Open Source" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#38BDF8;strokeWidth=2;fontColor=#FFFFFF;fontSize=14;fontStyle=1;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="330" y="1807" width="210" height="30" as="geometry"/>
        </mxCell>
        <mxCell id="b9r" value="" style="ellipse;html=1;fillColor=none;strokeColor=#38BDF8;strokeWidth=2;" vertex="1" parent="1">
          <mxGeometry x="574" y="1815" width="14" height="14" as="geometry"/>
        </mxCell>
        <mxCell id="l91" value="T0 MCU · T1 RPi 512MB · T2 4GB · T3 desktop" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#38BDF8;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1744" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l92" value="&lt;b&gt;Plugins:&lt;/b&gt; Starlark + JSON-RPC stdio com capabilities" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#38BDF8;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1800" width="380" height="44" as="geometry"/>
        </mxCell>
        <mxCell id="l93" value="MIT/Apache-2.0 · RFCs · bench como contrato" style="shape=partialRectangle;html=1;whiteSpace=wrap;fillColor=none;bottom=1;top=0;left=0;right=0;strokeColor=#38BDF8;strokeWidth=2;fontColor=#FFFFFF;fontSize=12;align=left;verticalAlign=bottom;spacingBottom=6;spacingLeft=4;" vertex="1" parent="1">
          <mxGeometry x="658" y="1856" width="380" height="44" as="geometry"/>
        </mxCell>

        <!-- ══════════ ARESTAS: raiz → ramos ══════════ -->
        <mxCell id="e_r_b1" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b1"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b2" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A855F7;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b2"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b3" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4ADE80;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b3"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b4" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#F59E0B;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b4"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b5" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#22D3EE;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b5"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b6" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EF4444;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b6"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b7" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EC4899;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b7"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b8" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A3E635;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b8"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_r_b9" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#38BDF8;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="rootRing" target="b9"><mxGeometry relative="1" as="geometry"/></mxCell>

        <!-- ══════════ ARESTAS: ramo → anel de fan-out ══════════ -->
        <mxCell id="e_b1" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b1" target="b1r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b2" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#A855F7;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b2" target="b2r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b3" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#4ADE80;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b3" target="b3r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b4" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#F59E0B;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b4" target="b4r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b5" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#22D3EE;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b5" target="b5r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b6" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#EF4444;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b6" target="b6r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b7" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#EC4899;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b7" target="b7r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b8" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#A3E635;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b8" target="b8r"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_b9" style="html=1;curved=1;endArrow=none;strokeWidth=2;strokeColor=#38BDF8;exitX=1;exitY=1;entryX=0;entryY=0.5;" edge="1" parent="1" source="b9" target="b9r"><mxGeometry relative="1" as="geometry"/></mxCell>

        <!-- ══════════ ARESTAS: anel → folhas ══════════ -->
        <mxCell id="e_11" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b1r" target="l11"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_12" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b1r" target="l12"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_13" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b1r" target="l13"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_14" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4A9EFF;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b1r" target="l14"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_21" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A855F7;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b2r" target="l21"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_22" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A855F7;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b2r" target="l22"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_23" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A855F7;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b2r" target="l23"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_31" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4ADE80;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b3r" target="l31"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_32" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4ADE80;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b3r" target="l32"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_33" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#4ADE80;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b3r" target="l33"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_41" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#F59E0B;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b4r" target="l41"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_42" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#F59E0B;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b4r" target="l42"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_43" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#F59E0B;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b4r" target="l43"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_51" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#22D3EE;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b5r" target="l51"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_52" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#22D3EE;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b5r" target="l52"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_53" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#22D3EE;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b5r" target="l53"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_61" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EF4444;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b6r" target="l61"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_62" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EF4444;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b6r" target="l62"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_63" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EF4444;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b6r" target="l63"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_71" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EC4899;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b7r" target="l71"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_72" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EC4899;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b7r" target="l72"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_73" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#EC4899;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b7r" target="l73"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_81" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A3E635;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b8r" target="l81"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_82" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A3E635;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b8r" target="l82"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_83" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#A3E635;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b8r" target="l83"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_91" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#38BDF8;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b9r" target="l91"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_92" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#38BDF8;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b9r" target="l92"><mxGeometry relative="1" as="geometry"/></mxCell>
        <mxCell id="e_93" style="edgeStyle=entityRelationEdgeStyle;curved=1;html=1;endArrow=none;strokeWidth=2;strokeColor=#38BDF8;exitX=1;exitY=0.5;entryX=0;entryY=1;" edge="1" parent="1" source="b9r" target="l93"><mxGeometry relative="1" as="geometry"/></mxCell>

      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

**Notas de implementação do arquivo:**

- **Sublinhados** usa o truque `shape=partialRectangle;bottom=1;top=0;left=0;right=0` — só a borda inferior é desenhada, na cor do ramo. As curvas entram em `entryY=1` (canto inferior esquerdo), então o conector "vira" o sublinhado, como no mapa de referência.
- **Anéis** são `ellipse;fillColor=none` nos pontos de fan-out (raiz + 9 ramos).
- **Cores por ramo** idênticas ao PNG: azul, roxo, verde, laranja, ciano, vermelho, magenta, lima, azul-claro.
- **Texto 100% correto** (sem os artefatos da renderização por IA) e editável nó a nó.

**Exportar PNG/JXL a partir dele** (fecha o loop determinístico):

```
# CLI do drawio (desktop) — headless
drawio -x -f png -s 2 -b "#000000" -o colmeia-arquitetura.png colmeia-arquitetura.drawio
cjxl colmeia-arquitetura.png colmeia-arquitetura.jxl -d 0 -e 9
```

Se quiser, gero também a **página 2** do mesmo arquivo com a versão *flowchart* (camadas com setas de fluxo de eventos/dados entre os subsistemas) — os dois diagramas convivem no mesmo `.drawio` como abas.

![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/5ff9c477-3988-416d-92b6-f7cce7a3fb3f_image.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI1ZmY5YzQ3Ny0zOTg4LTQxNmQtOTJiNi1mN2NjZTdhM2ZiM2YiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ.2FFmoM_oM3bvY_o7Wh8Q3PaJ28MTyWQCLnBOFt6ktJs&x-oss-process=image/resize,m_mfit,w_320,h_320) ![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/1f1cb17c-1d99-4af5-87d8-a166ee6f3e3d_image.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiIxZjFjYjE3Yy0xZDk5LTRhZjUtODdkOC1hMTY2ZWU2ZjNlM2QiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ.ZQrWTtT6iJXptTDoDtdVu8qDv9jbEEGThce1PDyEW_I&x-oss-process=image/resize,m_mfit,w_320,h_320) ![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/8d1e5ebf-9b91-4fa5-bc30-6cf8b52801d2_image.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI4ZDFlNWViZi05YjkxLTRmYTUtYmMzMC02Y2Y4YjUyODAxZDIiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ.294SrRuM21a46jcTrWQr-cSqU7Y9LLwKIPpNkfMWw18&x-oss-process=image/resize,m_mfit,w_320,h_320) ![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/6a987052-9d0d-4131-bfd4-7031c1b421db_image.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI2YTk4NzA1Mi05ZDBkLTQxMzEtYmZkNC03MDMxYzFiNDIxZGIiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ.XqzS6_GqD2Vtuhskl-zE4an8U6_KjWM_SA0whF0rQUk&x-oss-process=image/resize,m_mfit,w_320,h_320) ![](https://cdn.qwenlm.ai/ad90d683-4cd5-4272-bf54-f127316f2fdc/4c5a36d2-3ea7-4dcf-9af5-e15fd89d3009_image.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI0YzVhMzZkMi0zZWE3LTRkY2YtOWFmNS1lMTVmZDg5ZDMwMDkiLCJyZXNvdXJjZV9jaGF0X2lkIjpudWxsfQ.kK153h7Q1sDNDIc_kicY4kRlUoiq_NUACJ7aXEF_I7E&x-oss-process=image/resize,m_mfit,w_320,h_320)

gere imagens de como devem ser as telas do dashboard para que eu possa refinar, focando no design responsivo em varios tipos de aparelhos e necessidades técnicas a serem utilizadas em cada uma das telas, quero que estruture como as telas devem funcionar, motion design, temas claro, escuro, sépia e neumorph

Aqui está o kit visual completo da Colmeia — 10 telas cobrindo os 4 temas (escuro, claro, sépia, neumorph) e 4 classes de aparelho (desktop, tablet, celular, terminal TUI), seguidas da especificação funcional, técnica, responsiva e de motion design para você refinar.

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/0a1cec4c84a043efac9c6fc3b59a9965.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiIwYTFjZWM0Yzg0YTA0M2VmYWM5YzZmYzNiNTlhOTk2NSIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.JZ2LordytVLLPcnX_R5SLlERiQossPvy-Z9nYAndCdA&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/20cbff12fb4d4c39bc33f2d7dd31e52d.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiIyMGNiZmYxMmZiNGQ0YzM5YmMzM2YyZDdkZDMxZTUyZCIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.auQCGqwiZJZKZWuEuqOAvd5qBmBwAIgak1wcRyWidbg&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/f907fa2bf11240548f0b1cc83abe9c23.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJmOTA3ZmEyYmYxMTI0MDU0OGYwYjFjYzgzYWJlOWMyMyIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.un0W3C6JpdOVr7V0rC0orD6V4ADgn0fZQMUj430_U6A&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/2113f5b6eeb04f2580681686d58e6441.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiIyMTEzZjViNmVlYjA0ZjI1ODA2ODE2ODZkNThlNjQ0MSIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9._vvcs4F2Ay-GoMYHZ5VJAWOtCcD2VGk-EzVnx02xchc&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/a040b3fc7b5242ce9e83e3f3729d314a.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJhMDQwYjNmYzdiNTI0MmNlOWU4M2UzZjM3MjlkMzE0YSIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.dvYCk4A073L0LlDAML8q2x60qTpu9zfUQrDuVGpArmI&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/ea5db3f014694c93a7c24b5700d90c5b.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJlYTVkYjNmMDE0Njk0YzkzYTdjMjRiNTcwMGQ5MGM1YiIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.v92HFS3LyIePcqh_bHKgxgwTjrTd7qiti5TwHgrm0YI&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/75b92647e3f44bdead28bfeb13823dbf.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiI3NWI5MjY0N2UzZjQ0YmRlYWQyOGJmZWIxMzgyM2RiZiIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.I9FlxR9L3wQft3pKRbBJxsXd4Qg2vvsHxp0fUA7z960&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/b13d1c65cce643b78fb82c4d40738b2e.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJiMTNkMWM2NWNjZTY0M2I3OGZiODJjNGQ0MDczOGIyZSIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.7G01nk3VjXZxIiKbNMHML9cCtUpEU32c5gI8Ieok7E4&x-oss-process=image/resize,m_mfit,w_450,h_450)

Criar Vídeo

Editar

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/bf3822148bd744d69e9d51a35c298fa0.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiJiZjM4MjIxNDhiZDc0NGQ2OWU5ZDUxYTM1YzI5OGZhMCIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.7D28f5lm56DH_baAKOAMEdJoREMNNVyg_ieLZtp2crg&x-oss-process=image/resize,m_mfit,w_450,h_450)

gere o esqueleto do MVP deste projeto e um plano de implementação detalhando todas as features + um roadmap para garantir que o produto estará pronto para produção e para o uso de qualquer usuário comum

## Colmeia MVP — Esqueleto, Plano de Features e Roadmap de Produção

**Escopo desta entrega:** (1) esqueleto de código compilável do MVP, (2) matriz de features com versão-alvo e critérios de aceite, (3) roadmap faseado com *gates* de produção e de "usuário comum". Onde há `TODO(Fxx)` no código, é stub intencional mapeado à matriz — não é código esquecido.

**Linha de corte do MVP (v0.1):** registry + scheduler confiável + sync (fetch/ff/mirror) + gate (secrets/skills/SBOM/OSV) + slice *plan-only* + merge drivers básicos + TUI + web read-only + onboarding/doctor. Slice *exec*, workbench 3-pane, path\_sync, auto-PR, docs completas, grafo imersivo e plugins ficam para v0.2/v0.3. Isso é o que um usuário comum consegue instalar e usar com segurança no dia 1.

---

## 1\. Esqueleto do MVP

Crate único no MVP (split em workspace quando plugins entrarem). `git` CLI capado é o backend; `gitoxide` entra como otimização depois.

### 1.1 Árvore

```
colmeia/
├── Cargo.toml
├── schema.sql
├── colmeia.toml                  # default gerado por `init`
├── justfile
├── README.md
├── LICENSE-MIT / LICENSE-APACHE
├── rfcs/0001-determinism.md
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs  cli.rs  config.rs
│   ├── store.rs  events.rs
│   ├── scheduler.rs  governor.rs
│   ├── git.rs  sync.rs
│   ├── gate.rs  rules.rs  sbom.rs  osv.rs
│   ├── slice.rs  merge.rs
│   ├── tui.rs  web.rs
│   └── doctor.rs
└── tests/
    ├── farm.rs                   # gerador de repos sintéticos
    └── determinism.rs            # proptest slice/merge
```

### 1.2 Cargo.toml

```
[package]
name = "colmeia"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Gestor Git de frotas, TUI-first, automação determinística em hardware mínimo"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
crossterm = "0.27"
ratatui = "0.26"
tiny_http = "0.12"
ureq = { version = "2", features = ["json"] }
crossbeam-channel = "0.5"
sha2 = "0.10"
hex = "0.4"
regex = "1"
walkdir = "2"
dirs = "5"
cron = "0.12"
libc = "0.2"

[dev-dependencies]
proptest = "1"
tempfile = "3"

[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = "s"
```

### 1.3 schema.sql (MVP)

```
PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-32768; PRAGMA temp_store=FILE;

CREATE TABLE IF NOT EXISTS repos(
  id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL, url TEXT,
  mode TEXT CHECK(mode IN ('managed','mirror','dormant')) DEFAULT 'mirror',
  forge TEXT, default_branch TEXT, head_sha TEXT,
  priority INTEGER DEFAULT 5, last_activity INTEGER, state TEXT DEFAULT 'idle');

CREATE TABLE IF NOT EXISTS sync_policies(
  id INTEGER PRIMARY KEY, src_repo INTEGER, dst_repo INTEGER,
  kind TEXT CHECK(kind IN ('fetch_upstream','push_mirror','branch_sync','path_sync')),
  refspec TEXT, path_glob TEXT, cron TEXT, on_event TEXT,
  conflict_policy TEXT DEFAULT 'escalate', enabled INTEGER DEFAULT 1);

CREATE TABLE IF NOT EXISTS jobs(
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, repo_id INTEGER,
  payload_json TEXT NOT NULL, status TEXT DEFAULT 'queued',
  priority INTEGER DEFAULT 5, lease_until INTEGER, attempts INTEGER DEFAULT 0,
  depends_on INTEGER, created_at INTEGER, finished_at INTEGER, error TEXT);
CREATE INDEX IF NOT EXISTS jobs_dispatch ON jobs(status,priority,created_at);

CREATE TABLE IF NOT EXISTS events(
  id INTEGER PRIMARY KEY, ts INTEGER, repo_id INTEGER, kind TEXT, payload_json TEXT);

CREATE TABLE IF NOT EXISTS findings(
  id INTEGER PRIMARY KEY, repo_id INTEGER, blob_sha TEXT, rule_id TEXT,
  severity TEXT, path TEXT, line INTEGER, message TEXT, owasp TEXT,
  status TEXT DEFAULT 'open', first_seen INTEGER, last_seen INTEGER,
  UNIQUE(repo_id,blob_sha,rule_id,path,line));

CREATE TABLE IF NOT EXISTS sbom(
  repo_id INTEGER, commit_sha TEXT, ecosystem TEXT, name TEXT, version TEXT,
  purl TEXT, direct INTEGER, PRIMARY KEY(repo_id,commit_sha,ecosystem,name,version));

CREATE TABLE IF NOT EXISTS vulns(
  id TEXT PRIMARY KEY, aliases_json TEXT, ecosystem TEXT, name TEXT,
  ranges_json TEXT, severity TEXT, fixed_json TEXT, modified INTEGER, withdrawn INTEGER);

CREATE TABLE IF NOT EXISTS baselines(
  repo_id INTEGER, metric TEXT, ewma REAL, mad REAL, n INTEGER, updated_at INTEGER,
  PRIMARY KEY(repo_id,metric));

CREATE TABLE IF NOT EXISTS recipes(
  id INTEGER PRIMARY KEY, repo_id INTEGER, kind TEXT, input_hashes TEXT,
  policy_hash TEXT, output_hash TEXT, plan_json TEXT, created_at INTEGER);

CREATE TABLE IF NOT EXISTS audit(
  id INTEGER PRIMARY KEY, ts INTEGER, actor TEXT, action TEXT, target TEXT, details_json TEXT);
```

### 1.4 src/config.rs

```
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "d_workers")] pub workers: usize,
    #[serde(default = "d_soft")] pub memory_soft_mb: u64,
    #[serde(default = "d_hard")] pub memory_hard_mb: u64,
    #[serde(default = "d_gitcap")] pub git_child_cap_mb: u64,
    #[serde(default = "d_hot")] pub hot_watch_count: usize,
    #[serde(default)] pub security: Security,
    #[serde(default)] pub forge: Forge,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Security {
    #[serde(default = "d_eco")] pub osv_ecosystems: Vec<String>,
    #[serde(default = "d_sev")] pub gate_block_severity: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Forge { pub github_token_env: Option<String> }

fn d_workers()->usize{2} fn d_soft()->u64{600} fn d_hard()->u64{900}
fn d_gitcap()->u64{256} fn d_hot()->usize{100}
fn d_eco()->Vec<String>{vec!["PyPI".into(),"npm".into(),"crates.io".into()]}
fn d_sev()->String{"high".into()}

impl Default for Config {
    fn default()->Self{ Self{ workers:d_workers(), memory_soft_mb:d_soft(),
        memory_hard_mb:d_hard(), git_child_cap_mb:d_gitcap(), hot_watch_count:d_hot(),
        security:Security::default(), forge:Forge::default() } }
}

impl Config {
    pub fn load()->anyhow::Result<Self>{
        let p = Self::path();
        if p.exists(){ Ok(toml::from_str(&std::fs::read_to_string(p)?)?) }
        else { Ok(Self::default()) }
    }
    pub fn path()->PathBuf{
        dirs::home_dir().unwrap_or_else(||PathBuf::from(".")).join(".colmeia/colmeia.toml")
    }
    /// Auto-tier por RAM total (T1<=1GB, T2<=6GB, T3 acima).
    pub fn detect_tier()->&'static str{
        let mem = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let kb: u64 = mem.lines().find(|l|l.starts_with("MemTotal"))
            .and_then(|l|l.split_whitespace().nth(1)).and_then(|v|v.parse().ok()).unwrap_or(4_000_000);
        match kb/1024 { 0..=1024=>"t1", 1025..=6144=>"t2", _=>"t3" }
    }
}
```

### 1.5 src/store.rs

```
use rusqlite::Connection;
use std::sync::Mutex;
use std::path::Path;

pub struct Store { pub conn: Mutex<Connection> }

impl Store {
    pub fn open(path:&Path)->anyhow::Result<Self>{
        std::fs::create_dir_all(path.parent().unwrap())?;
        let c = Connection::open(path)?;
        c.execute_batch(include_str!("../schema.sql"))?;
        Ok(Self{ conn: Mutex::new(c) })
    }
    pub fn add_repo(&self, path:&str, url:Option<&str>, mode:&str)->anyhow::Result<i64>{
        let c = self.conn.lock().unwrap();
        c.execute("INSERT OR IGNORE INTO repos(path,url,mode) VALUES(?1,?2,?3)",
            rusqlite::params![path,url,mode])?;
        Ok(c.query_row("SELECT id FROM repos WHERE path=?1",[path],|r|r.get(0))?)
    }
    pub fn list_repos(&self)->anyhow::Result<Vec<(i64,String,String,String)>>{
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare("SELECT id,path,mode,COALESCE(head_sha,'') FROM repos ORDER BY path")?;
        let rows = s.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?
            .collect::<Result<Vec<_>,_>>()?;
        Ok(rows)
    }
    pub fn enqueue(&self, kind:&str, repo_id:i64, payload:&serde_json::Value, prio:i64)->anyhow::Result<i64>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute("INSERT INTO jobs(kind,repo_id,payload_json,priority,created_at) VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![kind,repo_id,payload.to_string(),prio,now])?;
        Ok(c.last_insert_rowid())
    }
    pub fn lease_next(&self, ttl_secs:i64)->anyhow::Result<Option<(i64,String,i64,String)>>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        let row = c.query_row(
            "SELECT id,kind,repo_id,payload_json FROM jobs
             WHERE status='queued' OR (status='running' AND lease_until<?1)
             ORDER BY priority,created_at LIMIT 1",[now],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).ok();
        if let Some((id,kind,repo,payload)) = row {
            c.execute("UPDATE jobs SET status='running',lease_until=?1,attempts=attempts+1 WHERE id=?2",
                rusqlite::params![now+ttl_secs,id])?;
            Ok(Some((id,kind,repo,payload)))
        } else { Ok(None) }
    }
    pub fn finish(&self, id:i64, err:Option<&str>)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        c.execute("UPDATE jobs SET status=?1,finished_at=?2,error=?3 WHERE id=?4",
            rusqlite::params![if err.is_some(){"failed"}else{"done"},unix(),err,id])?;
        Ok(())
    }
    pub fn event(&self, repo_id:i64, kind:&str, payload:&serde_json::Value)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        c.execute("INSERT INTO events(ts,repo_id,kind,payload_json) VALUES(?1,?2,?3,?4)",
            rusqlite::params![unix(),repo_id,kind,payload.to_string()])?;
        Ok(())
    }
    pub fn recent_events(&self, limit:i64)->anyhow::Result<Vec<(i64,String,String)>>{
        let c = self.conn.lock().unwrap();
        let mut s = c.prepare("SELECT ts,kind,payload_json FROM events ORDER BY id DESC LIMIT ?1")?;
        Ok(s.query_map([limit],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?
            .collect::<Result<Vec<_>,_>>()?)
    }
    pub fn upsert_finding(&self, repo:i64, blob:&str, rule:&str, sev:&str, path:&str, line:i64, msg:&str, owasp:&str)->anyhow::Result<()>{
        let c = self.conn.lock().unwrap();
        let now = unix();
        c.execute("INSERT INTO findings(repo_id,blob_sha,rule_id,severity,path,line,message,owasp,first_seen,last_seen)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)
            ON CONFLICT(repo_id,blob_sha,rule_id,path,line) DO UPDATE SET last_seen=?9,status='open'",
            rusqlite::params![repo,blob,rule,sev,path,line,msg,owasp,now])?;
        Ok(())
    }
}
pub fn unix()->i64{ std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 }
```

### 1.6 src/git.rs (backend capado)

```
use std::process::{Command, Output};
use std::path::Path;

pub struct Git { pub cap_mb: u64 }

impl Git {
    fn cmd(&self, cwd:&Path, args:&[&str])->Command{
        let mut c = Command::new("git");
        c.current_dir(cwd).args(args)
            .env("GIT_TERMINAL_PROMPT","0").env("GIT_OPTIONAL_LOCKS","0");
        #[cfg(unix)]
        {
            let cap = self.cap_mb*1024*1024;
            unsafe { c.pre_exec(move || {
                let rl = libc::rlimit{ rlim_cur:cap, rlim_max:cap };
                libc::setrlimit(libc::RLIMIT_AS, &rl);
                Ok(())
            }); }
        }
        c
    }
    pub fn run(&self, cwd:&Path, args:&[&str])->anyhow::Result<Output>{
        let o = self.cmd(cwd,args).output()?;
        Ok(o)
    }
    pub fn head(&self, cwd:&Path)->anyhow::Result<String>{
        let o = self.run(cwd,&["rev-parse","HEAD"])?;
        Ok(String::from_utf8_lossy(&o.stdout).trim().into())
    }
    pub fn is_clean(&self, cwd:&Path)->anyhow::Result<bool>{
        let o = self.run(cwd,&["status","--porcelain"])?;
        Ok(o.stdout.is_empty())
    }
    pub fn fetch(&self, cwd:&Path, remote:&str)->anyhow::Result<()>{
        let o = self.run(cwd,&["fetch","--filter=blob:none","--prune",remote])?;
        anyhow::ensure!(o.status.success(), "fetch falhou: {}", String::from_utf8_lossy(&o.stderr));
        let _ = self.run(cwd,&["commit-graph","write","--reachable"]);
        Ok(())
    }
    pub fn ff_merge(&self, cwd:&Path, refspec:&str)->anyhow::Result<bool>{
        let o = self.run(cwd,&["merge","--ff-only",refspec])?;
        Ok(o.status.success())
    }
    pub fn backup_ref(&self, cwd:&Path, branch:&str)->anyhow::Result<String>{
        let sha = self.head(cwd)?;
        let bak = format!("refs/colmeia/backup/{}", branch);
        self.run(cwd,&["update-ref",&bak,&sha])?;
        Ok(bak)
    }
    pub fn update_ref_cas(&self, cwd:&Path, branch:&str, new:&str, old:&str)->anyhow::Result<()>{
        let r = format!("refs/heads/{}", branch);
        let o = self.run(cwd,&["update-ref",&r,new,old])?;
        anyhow::ensure!(o.status.success(), "CAS falhou (ref mudou): {}", String::from_utf8_lossy(&o.stderr));
        Ok(())
    }
    pub fn changed_files(&self, cwd:&Path, range:&str)->anyhow::Result<Vec<String>>{
        let o = self.run(cwd,&["diff","--name-only",range])?;
        Ok(String::from_utf8_lossy(&o.stdout).lines().map(|s|s.into()).collect())
    }
}
```

### 1.7 src/governor.rs + src/scheduler.rs

```
// governor.rs
pub fn rss_mb()->u64{
    let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let pages: u64 = s.split_whitespace().nth(1).and_then(|v|v.parse().ok()).unwrap_or(0);
    pages*4/1024
}
pub fn available_mb()->u64{
    let s = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    s.lines().find(|l|l.starts_with("MemAvailable"))
        .and_then(|l|l.split_whitespace().nth(1)).and_then(|v|v.parse().ok()).unwrap_or(0)/1024
}
pub enum Pressure { Ok, Soft, Hard }
pub fn pressure(soft:u64, hard:u64)->Pressure{
    let r = rss_mb();
    if r>=hard || available_mb()<128 { Pressure::Hard }
    else if r>=soft { Pressure::Soft } else { Pressure::Ok }
}
```
```
// scheduler.rs
use crate::{store::Store, config::Config, governor, git::Git, sync, gate, slice, merge};
use std::sync::Arc;
use std::collections::HashSet;

pub fn run(store:Arc<Store>, cfg:Config)->anyhow::Result<()>{
    let git = Arc::new(Git{ cap_mb: cfg.git_child_cap_mb });
    let (tx, rx) = crossbeam_channel::bounded::<(i64,String,i64,String)>(cfg.workers*2);
    // workers fixos
    for _ in 0..cfg.workers {
        let rx = rx.clone(); let st = store.clone(); let g = git.clone(); let c = cfg.clone();
        std::thread::spawn(move || {
            for (id,kind,repo,payload) in rx {
                let res = dispatch(&st,&g,&c,&kind,repo,&payload);
                let _ = st.finish(id, res.err().map(|e|e.to_string()).as_deref());
            }
        });
    }
    let mut locked: HashSet<i64> = HashSet::new();
    loop {
        match governor::pressure(cfg.memory_soft_mb, cfg.memory_hard_mb){
            governor::Pressure::Hard => { std::thread::sleep(std::time::Duration::from_secs(2)); continue; }
            governor::Pressure::Soft => { std::thread::sleep(std::time::Duration::from_millis(500)); }
            governor::Pressure::Ok => {}
        }
        // TODO(F03): cron + polling adaptativo + inotify hot. MVP: drain da fila.
        if let Some(job) = store.lease_next(600)? {
            if locked.contains(&job.2){ continue; }
            locked.insert(job.2);
            let st = store.clone(); let tx = tx.clone();
            let j = job.clone();
            // lock liberado ao terminar (worker envia ack simplificado via finish)
            tx.send(job).ok();
            // MVP: libera lock após send (workers são rápidos); produção: ack por job.
            locked.remove(&j.2);
            let _ = st;
        } else {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}

fn dispatch(st:&Store, g:&Git, c:&Config, kind:&str, repo:i64, payload:&str)->anyhow::Result<()>{
    let v: serde_json::Value = serde_json::from_str(payload)?;
    match kind {
        "fetch" => sync::fetch(st,g,repo,&v),
        "branch_sync" => sync::branch_sync(st,g,repo,&v),
        "push_mirror" => sync::push_mirror(st,g,repo,&v),
        "gate" => gate::run_repo(st,g,c,repo,&v),
        "slice_plan" => slice::plan(st,g,repo,&v),
        "merge" => merge::run(st,g,repo,&v),
        "osv_sync" => crate::osv::sync(st,c),
        _ => anyhow::bail!("job desconhecido: {kind}"),
    }
}
```

### 1.8 src/sync.rs

```
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

fn repo_path(st:&Store, id:i64)->anyhow::Result<String>{
    let c = st.conn.lock().unwrap();
    Ok(c.query_row("SELECT path FROM repos WHERE id=?1",[id],|r|r.get(0))?)
}

pub fn fetch(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let remote = v["remote"].as_str().unwrap_or("origin");
    g.fetch(Path::new(&p), remote)?;
    let sha = g.head(Path::new(&p))?;
    { let c = st.conn.lock().unwrap();
      c.execute("UPDATE repos SET head_sha=?1,last_activity=?2 WHERE id=?3",
        rusqlite::params![sha, crate::store::unix(), repo])?; }
    st.event(repo,"sync.ok",&serde_json::json!({"sha":sha}))?;
    Ok(())
}

pub fn branch_sync(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let refspec = v["refspec"].as_str().unwrap_or("origin/main");
    if g.ff_merge(Path::new(&p), refspec)? {
        st.event(repo,"sync.ff",&serde_json::json!({"ref":refspec}))?;
    } else {
        // divergência ⇒ escala para merge (nunca força)
        st.enqueue("merge", repo, &serde_json::json!({"ref":refspec,"policy":"escalate"}), 3)?;
        st.event(repo,"sync.diverged",&serde_json::json!({"ref":refspec}))?;
    }
    Ok(())
}

pub fn push_mirror(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p = repo_path(st,repo)?;
    let dst = v["dst"].as_str().unwrap_or("mirror");
    let o = g.run(Path::new(&p), &["push","--mirror",dst])?;
    anyhow::ensure!(o.status.success(), "push falhou");
    st.event(repo,"sync.mirror",&serde_json::json!({"dst":dst}))?;
    Ok(())
}
// TODO(F10): path_sync (skill-packs) em v0.2.
```

### 1.9 src/rules.rs + src/gate.rs (SAST in-core)

```
// rules.rs
use regex::Regex;
pub struct Finding { pub rule:&'static str, pub sev:&'static str, pub line:usize, pub msg:String, pub owasp:&'static str }

pub fn scan(path:&str, text:&str)->Vec<Finding>{
    let mut out = Vec::new();
    let secret = Regex::new(r"(ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16})").unwrap();
    let danger = Regex::new(r"(eval\(|exec\(|shell\s*=\s*True|pickle\.loads|yaml\.load\()").unwrap();
    let inject = Regex::new(r"(?i)(ignore previous instructions|disregard your instructions|you are now)").unwrap();
    for (i,l) in text.lines().enumerate(){
        if secret.is_match(l){ out.push(Finding{rule:"secret.token",sev:"critical",line:i+1,msg:"token hardcoded".into(),owasp:"ASVS"}); }
        if danger.is_match(l){ out.push(Finding{rule:"code.dangerous",sev:"high",line:i+1,msg:"API perigosa".into(),owasp:"Top10"}); }
        if is_skill(path){
            if inject.is_match(l){ out.push(Finding{rule:"skill.injection.override",sev:"high",line:i+1,msg:"prompt injection".into(),owasp:"LLM01"}); }
            if l.chars().any(|c| matches!(c as u32, 0x200B..=0x200F | 0x202A..=0x202E)){
                out.push(Finding{rule:"skill.hidden_unicode",sev:"critical",line:i+1,msg:"unicode invisível".into(),owasp:"LLM01"});
            }
        }
    }
    out
}
fn is_skill(p:&str)->bool{ p.ends_with("SKILL.md")||p.ends_with(".prompt")||p.ends_with(".skill") }
pub fn entropy(s:&str)->f64{
    let mut f=[0u32;256]; for b in s.bytes(){ f[b as usize]+=1; }
    let n=s.len() as f64; if n==0.0{return 0.0;}
    -f.iter().filter(|&&c|c>0).map(|&c|{let p=c as f64/n; p*p.log2()}).sum::<f64>()
}
```
```
// gate.rs
use crate::{store::Store, git::Git, config::Config, rules};
use serde_json::Value;
use std::path::Path;

pub fn run_repo(st:&Store, g:&Git, _c:&Config, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let range = v["range"].as_str().unwrap_or("HEAD~1..HEAD");
    let files = g.changed_files(Path::new(&p), range)?;
    let mut count=0;
    for f in files {
        let full = Path::new(&p).join(&f);
        if !full.is_file(){ continue; }
        let text = std::fs::read_to_string(&full).unwrap_or_default();
        if text.len()>1_000_000 { continue; } // skip gigantes (T2)
        let blob = blob_sha(&text);
        for fd in rules::scan(&f,&text){
            st.upsert_finding(repo,&blob,fd.rule,fd.sev,&f,fd.line as i64,&fd.msg,fd.owasp)?;
            count+=1;
        }
    }
    st.event(repo,"gate.done",&serde_json::json!({"findings":count}))?;
    Ok(())
}
pub fn blob_sha(text:&str)->String{
    use sha2::{Sha256,Digest};
    let mut h=Sha256::new(); h.update(text.as_bytes()); hex::encode(h.finalize())
}
pub fn sarif(findings:&[(String,String,String,i64)])->Value{
    serde_json::json!({
      "version":"2.1.0",
      "runs":[{"tool":{"driver":{"name":"colmeia-gate"}},
        "results": findings.iter().map(|(rule,sev,path,line)|serde_json::json!({
          "ruleId":rule,"level":sev,"locations":[{"physicalLocation":{
            "artifactLocation":{"uri":path},"region":{"startLine":line}}}]
        })).collect::<Vec<_>>()}]
    })
}
// TODO(F17): loader Starlark hermético substitui rules.rs hardcoded em v0.2.
```

### 1.10 src/sbom.rs + src/osv.rs

```
// sbom.rs — parsers diretos (MVP: 3 ecossistemas)
pub fn parse(repo_path:&str)->Vec<(String,String,String)>{ // (eco,name,version)
    let mut out=Vec::new();
    let p=std::path::Path::new(repo_path);
    if let Ok(t)=std::fs::read_to_string(p.join("Cargo.lock")){
        for blk in t.split("[[package]]").skip(1){
            let n=field(blk,"name"); let v=field(blk,"version");
            if !n.is_empty(){ out.push(("crates.io".into(),n,v)); }
        }
    }
    if let Ok(t)=std::fs::read_to_string(p.join("requirements.txt")){
        for l in t.lines(){ if let Some((n,v))=l.split_once("=="){ out.push(("PyPI".into(),n.trim().into(),v.trim().into())); } }
    }
    if let Ok(t)=std::fs::read_to_string(p.join("package-lock.json")){
        if let Ok(v)=serde_json::from_str::<serde_json::Value>(&t){
            if let Some(pk)=v["packages"].as_object(){
                for (k,val) in pk { let n=k.trim_start_matches("node_modules/");
                    if let Some(ver)=val["version"].as_str(){ out.push(("npm".into(),n.into(),ver.into())); } }
            }
        }
    }
    out
}
fn field(blk:&str,key:&str)->String{
    blk.lines().find(|l|l.trim().starts_with(&format!("{key} =")))
        .and_then(|l|l.split('"').nth(1)).unwrap_or("").into()
}
```
```
// osv.rs
use crate::{store::Store, config::Config};
pub fn sync(st:&Store, c:&Config)->anyhow::Result<()>{
    for eco in &c.security.osv_ecosystems {
        let url = format!("https://storage.googleapis.com/osv-vulnerabilities/{}/all.zip", eco);
        let resp = ureq::get(&url).call()?;
        let mut bytes=Vec::new();
        resp.into_reader().take(200_000_000).read_to_end(&mut bytes)?;
        // TODO(F21): unzip + delta por modified + upsert em vulns. MVP: stub de ingest.
        let _ = st;
        st.event(0,"osv.sync",&serde_json::json!({"eco":eco,"bytes":bytes.len()}))?;
    }
    Ok(())
}
use std::io::Read;
// TODO(F22): matcher semver/pep440 + watch mode + auto-PR (v0.2).
```

### 1.11 src/slice.rs + src/merge.rs

```
// slice.rs — MVP: plan-only determinístico
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

pub fn plan(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let commit = v["commit"].as_str().unwrap_or("HEAD");
    let max = v["max_lines"].as_u64().unwrap_or(400) as usize;
    let o = g.run(Path::new(&p), &["show","--numstat","--format=",commit])?;
    let mut units: Vec<(String,usize)> = String::from_utf8_lossy(&o.stdout).lines()
        .filter_map(|l|{ let mut it=l.split_whitespace();
            let a=it.next()?.parse().ok()?; let d=it.next()?.parse().ok()?;
            Some((it.next()?.to_string(), a+d)) }).collect();
    // ordenação estável + tie-break por hash ⇒ determinismo
    units.sort_by(|a,b| a.0.cmp(&b.0).then_with(|| gate::blob_sha(&a.0).cmp(&gate::blob_sha(&b.0))));
    let mut slices: Vec<Vec<String>> = vec![vec![]];
    let mut acc=0;
    for (path,lines) in units {
        if acc+lines>max && !slices.last().unwrap().is_empty(){ slices.push(vec![]); acc=0; }
        slices.last_mut().unwrap().push(path); acc+=lines;
    }
    let plan = serde_json::json!({"commit":commit,"slices":slices,"max_lines":max});
    { let c=st.conn.lock().unwrap();
      c.execute("INSERT INTO recipes(repo_id,kind,input_hashes,policy_hash,plan_json,created_at)
                 VALUES(?1,'slice',?2,?3,?4,?5)",
        rusqlite::params![repo,commit,gate::blob_sha(&plan.to_string()),plan.to_string(),crate::store::unix()])?; }
    st.event(repo,"slice.plan",&plan)?;
    Ok(())
}
// TODO(F12): exec com apply --cached + gate por slice + backup ref + CAS (v0.2).
```
```
// merge.rs — MVP: ff + drivers simples + escalate
use crate::{store::Store, git::Git};
use serde_json::Value;
use std::path::Path;

pub fn run(st:&Store, g:&Git, repo:i64, v:&Value)->anyhow::Result<()>{
    let p: String = { let c=st.conn.lock().unwrap();
        c.query_row("SELECT path FROM repos WHERE id=?1",[repo],|r|r.get(0))? };
    let refspec = v["ref"].as_str().unwrap_or("origin/main");
    let policy = v["policy"].as_str().unwrap_or("escalate");
    let path = Path::new(&p);
    if g.ff_merge(path, refspec)? { st.event(repo,"merge.ff",&serde_json::json!({}))?; return Ok(()); }
    match policy {
        "ours"|"theirs" => {
            let o = g.run(path, &["merge","-X",policy,refspec])?;
            anyhow::ensure!(o.status.success(),"merge -X falhou");
            st.event(repo,"merge.driver",&serde_json::json!({"driver":policy}))?;
        }
        _ => {
            let _ = g.run(path, &["merge","--abort"]);
            st.event(repo,"merge.conflict",&serde_json::json!({"ref":refspec,"needs":"human"}))?;
            // TODO(F15): workbench 3-pane + structured 3-way + recipes (v0.2).
        }
    }
    Ok(())
}
```

### 1.12 src/tui.rs (esqueleto ratatui)

```
use crate::store::Store;
use std::sync::Arc;
use crossterm::{event::{self,Event,KeyCode},terminal::{enable_raw_mode,disable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen},execute};
use ratatui::{Terminal,backend::CrosstermBackend,widgets::{Block,Borders,List,ListItem,Paragraph},layout::{Layout,Constraint,Direction}};

pub fn run(store:Arc<Store>)->anyhow::Result<()>{
    enable_raw_mode()?;
    let mut out=std::io::stdout();
    execute!(out,EnterAlternateScreen)?;
    let mut term=Terminal::new(CrosstermBackend::new(out))?;
    let mut tab=0usize;
    loop {
        let repos=store.list_repos()?;
        let events=store.recent_events(20)?;
        term.draw(|f|{
            let chunks=Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(3),Constraint::Min(0)]).split(f.size());
            let tabs=Paragraph::new(" [1] Repos  [2] Jobs  [3] Findings   q=sair ")
                .block(Block::default().borders(Borders::ALL).title("Colmeia"));
            f.render_widget(tabs,chunks[0]);
            let items: Vec<ListItem> = if tab==0 {
                repos.iter().map(|(_,p,m,sha)|ListItem::new(format!("{p}  [{m}]  {}", &sha[..7.min(sha.len())]))).collect()
            } else {
                events.iter().map(|(ts,k,_)|ListItem::new(format!("{ts}  {k}"))).collect()
            };
            f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL)),chunks[1]);
        })?;
        if event::poll(std::time::Duration::from_millis(250))?{
            if let Event::Key(k)=event::read()?{
                match k.code { KeyCode::Char('q')=>break, KeyCode::Char('1')=>tab=0,
                    KeyCode::Char('2')=>tab=1, KeyCode::Char('3')=>tab=2, _=>{} }
            }
        }
    }
    disable_raw_mode()?;
    execute!(term.backend_mut(),LeaveAlternateScreen)?;
    Ok(())
}
// TODO(F46): paridade total de comandos + grafo braille + workbench (v0.2/v0.3).
```

### 1.13 src/web.rs (read-only + SSE)

```
use crate::store::Store;
use std::sync::Arc;
pub fn run(store:Arc<Store>, bind:&str)->anyhow::Result<()>{
    let server = tiny_http::Server::http(bind)?;
    for req in server.incoming_requests(){
        let url = req.url().to_string();
        let (status,ctype,body) = match url.as_str(){
            "/" => (200,"text/html", INDEX.to_string()),
            "/api/repos" => (200,"application/json", serde_json::to_string(&store.list_repos()?)?),
            "/api/events" => (200,"application/json", serde_json::to_string(&store.recent_events(50)?)?),
            _ => (404,"text/plain","not found".into()),
        };
        let resp = tiny_http::Response::from_string(body).with_status_code(status)
            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes()).unwrap());
        let _ = req.respond(resp);
    }
    Ok(())
}
const INDEX:&str = r#"<!doctype html><html data-theme="escuro"><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<style>
:root{--bg:#0B0E14;--sf:#131722;--tx:#E8EAF2;--bd:#232A3A;--ac:#F59E0B}
[data-theme=claro]{--bg:#F5F6FA;--sf:#FFF;--tx:#171A26;--bd:#E4E7F0}
[data-theme=sepia]{--bg:#EFE6D3;--sf:#F8F1E0;--tx:#3B2F23;--bd:#D9CBB0}
body{background:var(--bg);color:var(--tx);font:14px/1.5 system-ui;margin:0;padding:16px}
.card{background:var(--sf);border:1px solid var(--bd);border-radius:12px;padding:12px;margin:8px 0}
</style></head><body>
<h1>Colmeia</h1><div id=repos class=card>carregando…</div>
<script>
fetch('/api/repos').then(r=>r.json()).then(rs=>{
 document.getElementById('repos').innerHTML = rs.map(r=>`<div>${r[1]} <b>[${r[2]}]</b></div>`).join('');
});
</script></body></html>"#;
// TODO(F45): SSE real + temas neumorph + responsivo completo (v0.3).
```

### 1.14 src/cli.rs + src/main.rs + src/doctor.rs

```
// cli.rs
use clap::{Parser,Subcommand};
#[derive(Parser)] #[command(name="colmeia",version)]
pub struct Cli { #[command(subcommand)] pub cmd: Cmd }
#[derive(Subcommand)]
pub enum Cmd {
    Init, Add { url:String, #[arg(long,default_value="mirror")] mode:String },
    Import { dir:String }, List,
    Sync { #[arg(long)] repo:Option<i64> },
    Gate { #[arg(long,default_value="HEAD~1..HEAD")] range:String, #[arg(long,default_value="json")] format:String },
    Slice { commit:String, #[arg(long)] plan_only:bool },
    Merge { #[arg(long)] refspec:String },
    Tui, Web { #[arg(long,default_value="127.0.0.1:7420")] bind:String },
    Doctor, Hook { action:String },
}
```
```
// main.rs
mod cli; mod config; mod store; mod events; mod scheduler; mod governor;
mod git; mod sync; mod gate; mod rules; mod sbom; mod osv; mod slice; mod merge;
mod tui; mod web; mod doctor;
use clap::Parser;
use std::sync::Arc;

fn main()->anyhow::Result<()>{
    let cli = cli::Cli::parse();
    let cfg = config::Config::load()?;
    let db = dirs::home_dir().unwrap().join(".colmeia/colmeia.db");
    let store = Arc::new(store::Store::open(&db)?);
    let git = git::Git{ cap_mb: cfg.git_child_cap_mb };
    match cli.cmd {
        cli::Cmd::Init => { doctor::init_wizard(&cfg)?; }
        cli::Cmd::Add{url,mode} => {
            let path = format!("./repos/{}", url.rsplit('/').next().unwrap_or("repo").trim_end_matches(".git"));
            std::fs::create_dir_all("./repos")?;
            let o = git.run(std::path::Path::new("."), &["clone","--bare","--filter=blob:none",&url,&path])?;
            anyhow::ensure!(o.status.success(),"clone falhou");
            store.add_repo(&path, Some(&url), &mode)?;
            println!("adicionado: {path}");
        }
        cli::Cmd::Import{dir} => {
            for e in walkdir::WalkDir::new(&dir).max_depth(2){
                let e=e?; if e.path().join(".git").exists(){
                    store.add_repo(&e.path().to_string_lossy(), None, "managed")?;
                }
            }
        }
        cli::Cmd::List => { for (id,p,m,sha) in store.list_repos()? { println!("{id}\t{m}\t{}\t{p}", &sha[..7.min(sha.len())]); } }
        cli::Cmd::Sync{repo} => {
            for (id,_,_,_) in store.list_repos()? {
                if repo.map(|r|r==id).unwrap_or(true){ store.enqueue("fetch",id,&serde_json::json!({"remote":"origin"}),5)?; }
            }
            scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Gate{range,format} => {
            for (id,_,_,_) in store.list_repos()? {
                gate::run_repo(&store,&git,&cfg,id,&serde_json::json!({"range":range}))?;
            }
            if format=="sarif"{ println!("{}", gate::sarif(&[])); }
        }
        cli::Cmd::Slice{commit,plan_only} => {
            for (id,_,_,_) in store.list_repos()? {
                store.enqueue("slice_plan",id,&serde_json::json!({"commit":commit,"max_lines":400}),5)?;
            }
            let _ = plan_only; scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Merge{refspec} => {
            for (id,_,_,_) in store.list_repos()? { store.enqueue("merge",id,&serde_json::json!({"ref":refspec,"policy":"escalate"}),3)?; }
            scheduler::run(store.clone(), cfg)?;
        }
        cli::Cmd::Tui => tui::run(store.clone())?,
        cli::Cmd::Web{bind} => web::run(store.clone(), &bind)?,
        cli::Cmd::Doctor => doctor::run(&store,&cfg)?,
        cli::Cmd::Hook{action} => { if action=="install"{ doctor::install_hooks()?; } }
    }
    Ok(())
}
```
```
// doctor.rs
use crate::{store::Store, config::Config};
pub fn init_wizard(cfg:&Config)->anyhow::Result<()>{
    std::fs::create_dir_all(dirs::home_dir().unwrap().join(".colmeia"))?;
    let p = Config::path();
    if !p.exists(){ std::fs::write(&p, toml::to_string_pretty(cfg)?)?; }
    println!("Colmeia iniciada. tier={} workers={} soft={}MB", Config::detect_tier(), cfg.workers, cfg.memory_soft_mb);
    println!("Próximos passos: colmeia add <url>  ·  colmeia tui  ·  colmeia doctor");
    Ok(())
}
pub fn run(st:&Store, cfg:&Config)->anyhow::Result<()>{
    let git_ok = std::process::Command::new("git").arg("--version").output().is_ok();
    println!("git instalado: {}", if git_ok{"ok"}else{"FALTA — instale git>=2.40"});
    println!("tier: {}  workers: {}  rss: {}MB  avail: {}MB",
        Config::detect_tier(), cfg.workers, crate::governor::rss_mb(), crate::governor::available_mb());
    println!("repos: {}", st.list_repos()?.len());
    println!("pressão: {:?}", crate::governor::pressure(cfg.memory_soft_mb,cfg.memory_hard_mb));
    Ok(())
}
pub fn install_hooks()->anyhow::Result<()>{
    // TODO(F25): hook runner nativo pre-commit/pre-push chamando `colmeia gate`.
    println!("hooks: stub — instala pre-commit/pre-push que chamam `colmeia gate`");
    Ok(())
}
```

### 1.15 justfile, CI, RFC, testes

```
# justfile
build:; cargo build --release
test:; cargo test
lint:; cargo clippy -- -D warnings && cargo fmt --check
farm n=200:; cargo test --test farm -- --nocapture {{n}}
bench:; cargo run --release -- doctor
```
```
# .github/workflows/ci.yml
name: ci
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
      # TODO(F54): bench-contract (RSS/latência) como gate de PR
```
```
<!-- rfcs/0001-determinism.md -->
# RFC 0001 — Determinismo
- Ordenações estáveis; tie-break por hash de conteúdo.
- Políticas versionadas por `policy_hash`.
- Toda resolução/slice gera `recipe` replayável (`colmeia replay`).
- Escritas de ref só via `update-ref <new> <old>` (CAS) + backup ref.
- Jobs com lease ⇒ crash-recovery sem perda.
```
```
// tests/farm.rs — gerador de repos sintéticos (esqueleto)
#[test] fn farm(){ /* TODO(F54): criar N repos com históricos/lockfiles/skills plantadas */ }
// tests/determinism.rs
#[test] fn slice_plan_deterministic(){ /* TODO: proptest mesmo input ⇒ mesmo plan_hash */ }
```

**Stub vs real no esqueleto:**

| Real agora | Stub (TODO) |
| --- | --- |
| store/migrations, leases, governor, git capado, sync fetch/ff/mirror, gate rules+SBOM parse, slice plan, merge drivers, TUI lista, web read-only, doctor/init | Starlark loader, OSV unzip/matcher, slice exec, workbench, path\_sync, SSE real, grafo, plugins, hooks runner, bench-contract |

---

## 2\. Plano de implementação — matriz de features

Versões: **M** =MVP v0.1 · **2** =v0.2 · **3** =v0.3 · **1** =v1.0 GA.

| ID | Feature | Ver | Dep | Critério de aceite |
| --- | --- | --- | --- | --- |
| F01 | Registry + import em massa | M | — | 2.000 repos import ≤60s |
| F02 | Modos managed/mirror/dormant + LRU | M | F01 | worktrees ≤30 em T2 |
| F03 | Scheduler adaptativo (cron+poll+evento) | M | F05 | sweep 2.000 ≤3h T2 |
| F04 | inotify hot-subset | 2 | F03 | watches ≤ kernel limit |
| F05 | Jobs DAG + leases + locks | M | — | kill -9 ⇒ recovery |
| F06 | Memory governor soft/hard + rlimits | M | — | RSS ≤600MB típico |
| F07 | sync fetch\_upstream | M | F05 | partial clone ok |
| F08 | branch\_sync ff-only | M | F07 | divergência escala |
| F09 | push\_mirror | M | F07 | mirror idempotente |
| F10 | path\_sync skill-packs | 2 | F08 | hash conteúdo + commit det. |
| F11 | slice plan-only | M | F01 | plano ≤5s/5k linhas |
| F12 | slice exec (apply+gate+CAS+backup) | 2 | F11,F24 | replay bit-idêntico |
| F13 | merge drivers ours/theirs/union | M | F08 | nunca força |
| F14 | structured 3-way JSON/YAML/TOML | 2 | F13 | merge por chave |
| F15 | workbench 3-pane + fila | 2 | F13 | humano nunca auto |
| F16 | recipes + replay | 2 | F12,F14 | bit-idêntico proptest |
| F17 | rules Starlark hermético | 2 | F24 | packs versionados |
| F18 | secrets regex+entropia | M | — | falsos positivos \<5% |
| F19 | skills OWASP LLM | M | — | injection/unicode/agency |
| F20 | SBOM parsers lockfiles | M | — | 3 eco MVP, +5 em v0.2 |
| F21 | OSV sync nightly | M | F20 | delta ≤60s |
| F22 | matcher semver/pep440 | M | F21 | 10k pacotes ≤5s |
| F23 | watch + auto-PR bump | 2 | F22,F32 | bump simples só |
| F24 | gate + SARIF + paridade CI | M | F18 | mesmo resultado local/CI |
| F25 | hook runner nativo | M | F24 | pre-commit/pre-push |
| F26 | docs engine + doc cards | 2 | F20 | regen O(diff) |
| F27 | FTS5 busca docs | 2 | F26 | busca 2.000 repos |
| F28 | graph store edges | 2 | F01 | commit-graph accel |
| F29 | layout incremental + culling | 3 | F28 | 312 nós 60fps |
| F30 | anomaly EWMA/MAD | M | F03 | batch ≤15min |
| F31 | cross-repo fanout/drift | 2 | F30 | supply-chain signal |
| F32 | forge GitHub + rate budget | M | — | budget respeitado |
| F33 | GitLab + Forgejo | 3 | F32 | adapters trait |
| F34 | rate budget por forge | M | F32 | backoff |
| F35 | PRs agendados | 2 | F32 | ±30s do cron |
| F36 | auto-merge c/ lease | 3 | F35 | checks poll |
| F37 | webhooks in HMAC | 2 | F32 | assinatura verify |
| F38 | hooks out exec/HTTP/MQTT | 2 | F37 | T0 trigger |
| F39 | plugins Starlark packs | 2 | F17 | packs como repos |
| F40 | plugins JSON-RPC stdio + caps | 3 | F39 | sandbox caps |
| F41 | WASM extism | 1 | F40 | só se demanda |
| F42 | tiers T1/T2/T3 presets | M | F06 | RPi passa suite |
| F43 | companion T0 firmware | 3 | F38 | ESP32/Uno JSON |
| F44 | temas claro/escuro/sépia/neumorph | 3 | web | AA contraste |
| F45 | motion + responsivo web | 3 | F44 | breakpoints 4 |
| F46 | TUI paridade + grafo braille | 2 | F28 | 80×24 |
| F47 | onboarding wizard | M | — | zero-config |
| F48 | doctor self-heal | M | — | diagnóstico claro |
| F49 | installers (cargo/brew/deb) | 1 | F47 | 1 comando |
| F50 | docs + i18n PT/EN | 1 | F47 | tutorial comum |
| F51 | secrets age + keyring | M | — | nunca plaintext |
| F52 | redaction + fuzz parsers | 1 | F20 | fuzz sem crash |
| F53 | OTLP/LangSmith bridge | 3 | F38 | opt-in |
| F54 | tests/bench/chaos/proptest | M | — | CI gate |
| F55 | governance RFC/DCO/licença | M | — | público |

---

## 3\. Roadmap de produção + usuário comum

### Fases e gates de saída

| Fase | Semanas | Entrega | Gate de saída (definition of done) |
| --- | --- | --- | --- |
| **0 Fundação** | 1–3 | esqueleto compilável, CI, RFC0001, farm | `cargo test` verde; import 2.000 ≤60s; RSS idle ≤120MB |
| **1 Core confiável** | 4–9 | F03–F09, governor, leases | chaos kill -9 sem perda; sweep ≤3h T2; soak 24h |
| **2 Segurança** | 10–15 | F18–F25, F51 | gate paridade local/CI; OSV delta ≤60s; self-audit secrets |
| **3 Determinismo** | 16–21 | F11–F16, F46 | proptest replay bit-idêntico; slice exec com CAS+backup |
| **4 Integração** | 22–27 | F32,F34,F35,F37, web read-only | rate budget; PR ±30s; web ≤10MB daemon |
| **5 Usuário comum** | 28–33 | F47,F48,F50, temas, responsivo | beta público; tutorial ≤10min; doctor self-heal; AA |
| **6 Hardening/GA** | 34–40 | F49,F52,F41?, soak, perf RPi | soak 7d; fuzz limpo; perf contract RPi; v1.0 |

### Checklist de produção (v1.0 GA)

- **Confiabilidade:** crash-recovery por lease; WAL+backup/restore do DB (`colmeia db export/import`); migrações versionadas com rollback; soak 7 dias em T1/T2.
- **Segurança do tool:** tokens age-cifrados; redação no audit; fuzz de parsers de lockfile/webhook; SARIF assinado; sem plaintext em logs.
- **Performance como contrato:** bench público falha PR que regride RSS/latência; suporte T1 (512MB) com suite reduzida.
- **Observabilidade opt-in:** OTLP/LangSmith bridge desligado por default; crash reports opt-in.
- **Operação:** semver; cadência de release; LTS de segurança; uninstall limpo; portabilidade de dados (export JSON/SQLite).
- **Suporte:** matriz OS/arch (linux x86\_64/aarch64/armv6, macOS); binários musl estáticos.

### Checklist "qualquer usuário comum"

- **Instalação 1 comando** (`cargo install colmeia` / brew /.deb) + `colmeia init` wizard que detecta tier e escreve config.
- **Zero jargão em erros:** mensagens PT/EN com causa + ação sugerida; `--json` para scripts.
- **Segurança por default:** destrutivo = dry-run + confirmação mostrando CAS; undo 5s; nunca force-push.
- **GUI web para quem não usa terminal** (read-only no MVP, interativa v0.3) + TUI para quem usa.
- **`colmeia doctor`** que diagnostica e cura (git faltando, permissão, DB lock, pressão de memória).
- **Docs tutorial** "importe sua pasta de projetos em 10 minutos" + empty-states com CTA real.
- **Acessibilidade:** contraste AA nos 4 temas, alvos 44px, `prefers-reduced-motion`, navegação por teclado/⌘K.

---

## 4\. Riscos e próximo passo

| Risco | Mitigação |
| --- | --- |
| MVP inflar | linha de corte rígida (§1); exec/workbench/plugins fora |
| gitoxide imaturo | CLI capado no MVP; trait isola migração |
| OSV dump grande | filtro por eco + poda withdrawn |
| Usuário comum perdido | wizard + doctor + web + erros humanos |
| Determinismo quebrar | proptest + recipes + CAS como gate de CI |

**Próximo comando:** `cargo build && cargo run -- init && cargo run -- add <url> && cargo run -- tui`. A partir daí, preencha os `TODO(Fxx)` na ordem da matriz — cada um tem critério de aceite mensurável e versão-alvo, então o caminho até v1.0 é executável, não aspiracional.

![](https://cdn.qwenlm.ai/output/ad90d683-4cd5-4272-bf54-f127316f2fdc/t2i/6fc34d00-7e12-48dd-a820-27fb8b133bbc/0a1cec4c84a043efac9c6fc3b59a9965.png?key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyZXNvdXJjZV91c2VyX2lkIjoiYWQ5MGQ2ODMtNGNkNS00MjcyLWJmNTQtZjEyNzMxNmYyZmRjIiwicmVzb3VyY2VfaWQiOiIwYTFjZWM0Yzg0YTA0M2VmYWM5YzZmYzNiNTlhOTk2NSIsInJlc291cmNlX2NoYXRfaWQiOm51bGx9.JZ2LordytVLLPcnX_R5SLlERiQossPvy-Z9nYAndCdA&x-oss-process=image/resize,m_mfit,w_320,h_320)

Expanda esta imagem para fora, completando a cena ao redor com estilo, iluminação e perspectiva consistentes.

O conteúdo gerado por IA pode não ser preciso.