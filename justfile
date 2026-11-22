# Colmeia — Tasks and TDD Runner (justfile)

default:
    @just --list

# Compilar o binário em modo release
build:
    cargo build --release

# Compilar para verificação rápida de sintaxe e tipos
check:
    cargo check

# Executar toda a suíte de testes (TDD)
test:
    cargo test -- --nocapture

# Executar testes unitários específicos
test-unit:
    cargo test --test store_test --test rules_test --test slice_test --test sbom_test --test governor_test -- --nocapture

# Executar testes de integração (Git e determinismo)
test-integration:
    cargo test --test git_test --test determinism_test -- --nocapture

# Smoke test determinístico da Web GUI (estrutura, telas, temas, sintaxe JS)
test-web:
    node web/smoke-test.mjs

# Testes de integração com o diretório real de repositórios
# (somente leitura nos originais; override: COLMEIA_REPOS_TEST_DIR=/caminho)
test-repos:
    cargo test --test repos_dir_test -- --nocapture

# Teste de integração da API read-only de exploração (dev-server)
test-api:
    node scripts/api-test.mjs

# Teste do gate SAST (segredo plantado em repo temporário → finding secret.token)
test-gate:
    cargo test --test gate_test -- --nocapture

# Modo TDD contínuo: executa testes a cada arquivo alterado (red-green-refactor)
tdd:
    cargo watch -x "test -- --nocapture"

# Checagem estrita de linters e formatação de código
lint:
    cargo fmt --check
    cargo clippy -- -D warnings

# Aplicar formatação automática de código
fmt:
    cargo fmt

# Executa diagnóstico local
doctor:
    cargo run -- doctor

# Inicia a interface TUI interativa
tui:
    cargo run -- tui

# Inicia o servidor Web / SSE na porta 7420
web:
    cargo run -- web --bind 127.0.0.1:7420

# ------------------------------------------------------------------------------
# Comandos Container / Docker Compose
# ------------------------------------------------------------------------------

# Constrói as imagens de container do projeto
docker-build:
    docker compose build

# Executa a suíte de testes TDD dentro do container isolado
docker-test:
    docker compose run --rm colmeia-test

# Sobe o ambiente de desenvolvimento interativo em background
docker-up:
    docker compose up -d colmeia-dev

# Abre uma sessão bash interativa dentro do container de desenvolvimento
docker-shell:
    docker compose exec colmeia-dev bash

# Executa a interface TUI diretamente pelo container com alocação de terminal
docker-tui:
    docker compose run --rm colmeia-tui
