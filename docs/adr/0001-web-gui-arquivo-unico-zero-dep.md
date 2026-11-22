# ADR 0001 — Web GUI em arquivo único, zero dependências

- Status: Aceito
- Data: 2026-11-20

## Contexto

O contrato do projeto (hardware restrito) proíbe frameworks SPA pesados, exige
renderização server-side leve com SSE e limita o binário final. A Web GUI precisa
rodar em um daemon `tiny_http` embutido via `include_str!("../web/index.html")`
e continuar funcional offline.

## Decisão

Manter a Web GUI como **um único `web/index.html` autocontido** (HTML + CSS +
JS vanilla, sem build, sem npm, sem CDN obrigatório):

- Servido byte a byte pelo daemon Rust em produção.
- Servido pelo `scripts/dev-server.mjs` (Node puro, zero deps) apenas como preview local.
- Estado vivo via SSE (`/events`); o restante é estado client-side com seed
  determinística — a GUI é server-agnostic por design.
- Um único `requestAnimationFrame` para todos os canvases (grafo, branches, fluxo).

## Alternativas consideradas

- **React/Tailwind/Framer Motion** (stack sugerida numa das referências visuais):
  descartada — viola o contrato de leveza (Electron/Tauri/React/Three.js fora de escopo).
- **Templates server-side (maud)**: descartada por agora — o arquivo único já
  satisfaz "server-rendered leve + SSE" sem adicionar crate de template.
- **Split em múltiplos assets**: adiada; o arquivo único (~146 KB) ainda é
  barato de embedar e serve. Reavaliar se passar de ~300 KB.

## Consequências

- Zero build step; hot-reload é só recarregar o arquivo.
- Toda interação pesada (zoom, grafo, branching) é canvas 2D — OK para UHD 620.
- Testável com um smoke test estático determinístico (`web/smoke-test.mjs`).
