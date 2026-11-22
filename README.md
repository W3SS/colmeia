# 🐝 Colmeia — Gestor Git de Frotas & Automação Determinística

> **Gestor Git open-source, TUI-first com visualizador Web opcional, focado em automação determinística para hardware mínimo.**

O **Colmeia** foi concebido para gerenciar milhares de repositórios Git ("células") com workers locais ("abelhas") e um scheduler/dispatcher inteligente ("rainha"), operando com consumo estritamente controlado de memória em máquinas modestas (*"computador batata"* com 4 GB DDR3 sem GPU ou nós headless como Raspberry Pi Zero 2 W com 512 MB de RAM).

---

## 🚀 Ambiente Container de Desenvolvimento & TDD

O projeto inclui um ambiente containerizado pronto para desenvolvimento imediato, com todas as ferramentas de compilação, Rust toolchain estável, SQLite WAL, Git $\ge$ 2.40 e runners de teste configurados.

### 1. Inicializando via Docker Compose

```bash
# 1. Construir a imagem de desenvolvimento
docker compose build

# 2. Executar a suíte completa de testes TDD isolada no container
docker compose run --rm colmeia-test

# 3. Subir o ambiente de desenvolvimento em background
docker compose up -d colmeia-dev

# 4. Abrir shell interativo no container
docker compose exec colmeia-dev bash

# 5. Executar a interface TUI com terminal interativo
docker compose run --rm colmeia-tui
```

### 2. Integração com VS Code / Antigravity IDE (Dev Containers)

Abra a pasta do projeto no VS Code ou IDE compatível. O arquivo `.devcontainer/devcontainer.json` será detectado automaticamente, permitindo que você clique em **"Reopen in Container"** para obter:
* Rust Analyzer configurado.
* Clippy e Rustfmt no salvamento.
* Encaminhamento automático da porta `7420` (Servidor Web da Colmeia).

---

## 🧪 Prática de TDD (Test-Driven Development)

O projeto adota TDD rigoroso. Utilize o `justfile` para automatizar o ciclo *red-green-refactor*:

```bash
# Executar todos os testes com output em tempo real
just test

# Executar testes unitários específicos (store, rules, slice, sbom, governor)
just test-unit

# Executar testes de integração (Git real e RFC 0001 determinismo)
just test-integration

# Modo TDD contínuo: reexecuta os testes automaticamente a cada arquivo salvo
just tdd

# Validação de formatação e linters estritos
just lint
```

### Cobertura da Suíte de Testes:
1. `tests/store_test.rs`: Integridade do SQLite em WAL mode, fila de jobs, leases com timeout e recuperação automática contra crashes (*kill -9*).
2. `tests/rules_test.rs`: Análise estática in-core (SAST): detecção de tokens (`ghp_`, `sk-`, `AKIA`), código perigoso (`eval`, `pickle`), prompt injections em skills (OWASP LLM01) e caracteres Unicode invisíveis/bidi override.
3. `tests/slice_test.rs`: Divisão determinística de commits monolíticos de IA em fatias atômicas $\le$ 400 linhas.
4. `tests/sbom_test.rs`: Parsers diretos de `Cargo.lock`, `requirements.txt` e `package-lock.json` sem dependência de scanners externos.
5. `tests/governor_test.rs`: Verificação de limiares de memória (soft: 600 MB, hard: 900 MB) e auto-detecção de tiers.
6. `tests/git_test.rs`: Integração com repositórios Git reais temporários via `GitBackend` (commit, head SHA, numstat, fast-forward merge).
7. `tests/determinism_test.rs`: Contrato da **RFC 0001**: garantia de que mesmos inputs geram hashes e planos bit-idênticos.

---

## 📐 Tiers de Hardware

| Tier | Hardware | O que roda | Orçamento RSS | Concorrência |
| :--- | :--- | :--- | :--- | :--- |
| **T0 — Companion** | Arduino / ESP32 | Gatilho físico, display OLED, LED de alerta | < 40 KB | 0 |
| **T1 — Headless** | RPi Zero 2 W (512 MB) | Core + scheduler + sync + gates | $\le$ 180 MB | 1 worker |
| **T2 — Batata (Alvo)** | PC 4 GB DDR3 sem GPU | Core + TUI + Web mode + Scans + Slice + Merge | $\le$ 600 MB (Hard: 900 MB) | 2–4 workers |
| **T3 — Desktop** | 8 GB+ RAM | T2 com concorrência máxima | $\le$ 2 GB | 8 workers |

---

## 🛠 Comandos da CLI (`colmeia`)

| Comando | Descrição |
| :--- | :--- |
| `colmeia init` | Inicializa configuração e banco de dados |
| `colmeia add <url> [--mode mirror]` | Registra e clona repositório na frota |
| `colmeia import <dir>` | Importa em lote todos os repositórios Git de uma pasta |
| `colmeia list` | Lista todos os repositórios da frota e estados atuais |
| `colmeia sync [--repo <id>]` | Dispara sincronizações com políticas determinísticas |
| `colmeia gate [--range A..B]` | Executa auditoria SAST/Skills (saída JSON ou SARIF) |
| `colmeia slice <commit>` | Divide commits monolíticos em fatias atômicas |
| `colmeia merge --refspec <ref>` | Executa merge determinístico com recipes replayáveis |
| `colmeia tui` | Abre a interface de terminal (Ratatui) |
| `colmeia web [--bind 127.0.0.1:7420]` | Sobe a interface web leve server-rendered |
| `colmeiad` | Executa o dispatcher em segundo plano, verificando a fila a cada segundo |
| `colmeia doctor` | Autodiagnóstico de dependências e métricas |

O dashboard web usa HTML/CSS/JavaScript local, apresenta os temas Claro, Escuro,
Sépia e Neumorph e recebe snapshots incrementais por Server-Sent Events em
`/events`. Os dados de repositórios, jobs e eventos ficam disponíveis em
`/api/repos`, `/api/jobs` e `/api/events`.

Para iniciar o app neste checkout, rode `sh scripts/start-local.sh` e abra
`http://127.0.0.1:7420/`. O diretório de build fica no temporário do host,
pois montagens de repositórios podem não permitir executar binários. Abrir
`web/index.html` diretamente por `file://` não conecta à API.

Para mapear clones já existentes, use **Selecionar pasta**
para navegar pelo sistema de arquivos local e escolha **Mapear repositórios**.
Também é possível informar um caminho absoluto no campo. Pela CLI:

```bash
colmeia import ~/Ubuntu/home/Documents/repos
```

O mapeamento registra os caminhos no banco da Colmeia e lê o HEAD e a branch de cada
repositório. Ele não move, apaga nem altera os repositórios encontrados. A
seleção e o mapeamento pela web só aceitam o servidor ligado em localhost.
