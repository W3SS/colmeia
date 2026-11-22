# Colmeia — Getting started (English)

Colmeia is a Git-fleet manager: TUI-first, deterministic, and light enough
to run on minimum hardware.

## Requirements

- Git ≥ 2.40
- Rust ≥ 1.75 (build only)

## Install

```sh
cargo install --path .
colmeia init
```

## Quick tour

```sh
colmeia import ~/projects     # map existing repositories
colmeia list                  # list the fleet
colmeia sync                  # fetch-sync everything
colmeia gate --range HEAD~1..HEAD   # SAST over a commit range
colmeia tui                   # terminal UI (tabs 1/2/3, 'q' to quit)
colmeia web                   # web dashboard at http://127.0.0.1:7420
colmeia doctor --heal         # diagnostics + self-healing
```

## Secrets vault (F51)

```sh
export COLMEIA_PASSPHRASE='my-password'
colmeia vault set github-token --value ghp_xxx
colmeia vault get github-token
```

## Git hooks (F25)

```sh
colmeia hooks-install ~/projects/my-repo
```

## Observability

Set `COLMEIA_OTLP_ENDPOINT=http://collector:4318` to export OTLP/JSON events.
When unset, nothing is sent (zero cost by default).

## Hardware profiles

`colmeia init` asks for a profile when run in a real terminal:
`minimum` (1 worker, 96 MB), `recommended` (2 workers, 256 MB), or
`maximum` (4 workers, 512 MB). Non-interactive runs use auto-detection.
