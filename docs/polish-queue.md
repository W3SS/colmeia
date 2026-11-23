# Fila de polimento UI — dashboard premium

Loop de aprimoramento contínuo do frontend (`web/index.html`). Cada item é
pequeno, verificável e independente. **Regras do loop:**

1. Pegue o PRÓXIMO item `pendente` da lista (não pule).
2. Implemente em `web/index.html` (zero dependências — CSS/JS inline).
3. Valide: `node web/smoke-test.mjs` deve ficar 100% OK; se tocar em API,
   `npm run test:api` também.
4. Verificação visual quando possível: abrir `http://localhost:7100/` no
   navegador embutido e conferir o item (DOM/computed styles).
5. Marque o item `feito` com a data e o que foi alterado; commite com
   mensagem `polish(<id>): <resumo>` e dê push.
6. Se um item quebrar os testes, reverta e marque `bloqueado` com a causa.

Referência visual: dashboard premium tipo SaaS (sidebar escura com item ativo
em pílula, cards arredondados com sombras suaves em camadas, espaçamento
generoso, tipografia limpa, micro-interações discretas).

## Itens

| # | id | Item | Critério de aceite | Status |
|---|----|------|--------------------|--------|
| 1 | p01 | Sombras em camadas + raio refinado | `--shadow-card` por tema; card usa a sombra; `--r` 14px | ✅ feito 2026-11-23 |
| 2 | p02 | Rail ativo em pílula de acento | `.rail button.on` com bg de acento 30% + anel + glow | ✅ feito 2026-11-23 |
| 3 | p03 | Tipografia: antialiased + optimizeLegibility + títulos de card com peso/tracking | computed styles conferem | ✅ feito 2026-11-23 |
| 4 | p04 | Tabelas sem borda na última linha; foco de inputs com anel de acento; toast elevado | CSS aplicado; smoke OK | ✅ feito 2026-11-23 |
| 5 | p05 | KPI cards: número com gradiente sutil de acento + separador lateral com glow | `.kpi .num` e `::after` refinados | pendente |
| 6 | p06 | Drawer/modal: entrada com fade+slide suave; backdrop com blur progressivo | transições em `--enter`; backdrop-filter | pendente |
| 7 | p07 | Chips/filtros com estado ativo mais legível (bg acento 20% + borda acento) | `.chip.on` aplicado em todas as telas | pendente |
| 8 | p08 | Header: busca com ícone + placeholder de atalho (⌘K) estilo premium; pill do governor com dot pulsante | visual consistente | pendente |
| 9 | p09 | Empty states: ilustração/ícone grande + ação primária quando lista vazia (repos, PRs, docs) | telas sem dados mostram empty state | pendente |
| 10 | p10 | Skeleton loading: shimmer discreto nos cards enquanto `state.repos` carrega | shimmer via gradiente animado, respeita `--motion:0` | pendente |
| 11 | p11 | Tema claro: sombras ainda mais suaves + bordas com contraste refinado | audit visual no claro | pendente |
| 12 | p12 | Foco visível global consistente (anel de acento 3px em todos os interativos) | tab navigation confere | pendente |
| 13 | p13 | Responsividade mobile: bottom nav ou rail compacto com labels; cards em 1 coluna | 375px conferido | pendente |
| 14 | p14 | Motion design: easing `cubic-bezier(.2,.8,.2,1)` nas transições principais; respeito a prefers-reduced-motion | transições suaves | pendente |
| 15 | p15 | Detalhes finais: tooltips nos ícones do rail; títulos de seção com tracking; consistência de ícones (mesma espessura de traço) | audit final | pendente |

## Log de iterações

- **2026-11-23 — rodada manual (agente)**: p01–p04 implementados; smoke 58/58;
  verificação via DOM no navegador embutido (raio 14px confirmado após fix do
  fallback `--r` em `applyThemeVars`, que resetava para 12).
