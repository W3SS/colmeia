# docs/ — Decisões de arquitetura do projeto Colmeia

Esta pasta armazena as decisões de arquitetura (ADRs), levantamentos e
documentos vivos do projeto. O formato segue o espírito do
`rfcs/0001-determinismo.md`: decisões curtas, com contexto, alternativas e
consequências — reversíveis sempre que possível.

## Índice

| Documento | Tema | Status |
|---|---|---|
| [adr/0001-web-gui-arquivo-unico-zero-dep.md](adr/0001-web-gui-arquivo-unico-zero-dep.md) | Frontend em arquivo único, sem framework, servido por `include_str!` | Aceito |
| [adr/0002-sistema-de-temas-e-fontes.md](adr/0002-sistema-de-temas-e-fontes.md) | Motor de temas: 4 base + retrô + energia + 18 cyberpunk com fontes combinadas | Aceito |
| [adr/0003-menu-hamburguer-e-responsividade.md](adr/0003-menu-hamburguer-e-responsividade.md) | Navegação off-canvas com hambúrguer; breakpoints e bugfix 901–1180 px | Aceito |
| [adr/0004-automacao-de-testes.md](adr/0004-automacao-de-testes.md) | Smoke test da Web GUI + testes sobre o diretório real de repos | Aceito |
| [adr/0005-preview-dev-server.md](adr/0005-preview-dev-server.md) | Dev-server Node puro apenas para preview; produção continua no daemon Rust | Aceito |
| [FEATURES-PENDENTES.md](FEATURES-PENDENTES.md) | Levantamento F01–F55 e M0–M9 vs. implementação atual | Contínuo |

## Convenções

- Um ADR por decisão significativa; propostas ficam com status `Proposto`.
- Mudanças de decisão **não apagam** o ADR antigo — atualiza-se o status para
  `Superado` com link para o novo.
- Tudo aqui é leve em texto (Markdown), versionado junto com o código.
