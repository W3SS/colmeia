# ADR 0005 — Preview com dev-server Node puro (não é dependência de runtime)

- Status: Aceito
- Data: 2026-11-20

## Contexto

A Kimi Work precisa abrir a Web GUI como preview (`npm run dev`), mas o
contrato proíbe Node como dependência obrigatória de execução do produto.

## Decisão

- `scripts/dev-server.mjs`: servidor estático com **Node built-ins apenas**
  (`node:http`, `node:fs`), aceitando `--root/--host/--port`; sem
  `node_modules`, sem instalação.
- `package.json` existe **só para o preview de desenvolvimento**.
- Produção inalterada: o daemon Rust (`tiny_http`) serve o mesmo
  `web/index.html` embutido — nenhum caminho de runtime depende de Node.

## Consequências

- `npm run dev` sobe o preview; `npm run test:web` roda o smoke test.
- O CI valida o smoke test e um HTTP 200 do servidor (job `web-ui`).
