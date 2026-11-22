# Colmeia — Primeiros passos (pt-BR)

O Colmeia é um gestor Git de frotas: TUI-first, determinístico e leve o
suficiente para rodar em hardware mínimo.

## Requisitos

- Git ≥ 2.40
- Rust ≥ 1.75 (apenas para compilar)

## Instalação

```sh
cargo install --path .
colmeia init
```

## Tour rápido

```sh
colmeia import ~/projetos     # mapeia repositórios existentes
colmeia list                  # lista a frota
colmeia sync                  # sincroniza (fetch) todos
colmeia gate --range HEAD~1..HEAD   # SAST na faixa de commits
colmeia tui                   # interface de terminal (abas 1/2/3, 'q' sai)
colmeia web                   # painel web em http://127.0.0.1:7420
colmeia doctor --heal         # diagnóstico + autocorreção
```

## Cofre de segredos (F51)

```sh
export COLMEIA_PASSPHRASE='minha-senha'
colmeia vault set github-token --value ghp_xxx
colmeia vault get github-token
```

## Hooks git (F25)

```sh
colmeia hooks-install ~/projetos/meu-repo
```

## Temas de observabilidade

Defina `COLMEIA_OTLP_ENDPOINT=http://coletor:4318` para exportar eventos
OTLP/JSON. Sem a variável, nada é enviado (zero custo).

## Perfis de hardware

`colmeia init` pergunta o perfil quando executado em um terminal:
`minimum` (1 worker, 96 MB), `recommended` (2 workers, 256 MB) ou
`maximum` (4 workers, 512 MB). Fora de terminal, o auto-detectado é usado.
