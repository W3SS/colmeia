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
| 5 | p05 | KPI cards: número com gradiente sutil de acento + separador lateral com glow | `.kpi .num` e `::after` refinados | ✅ feito 2026-11-23 |
| 6 | p06 | Drawer/modal: entrada com fade+slide suave; backdrop com blur progressivo | transições em `--enter`; backdrop-filter | ✅ feito 2026-11-23 |
| 7 | p07 | Chips/filtros com estado ativo mais legível (bg acento 20% + borda acento) | `.chip.on` aplicado em todas as telas | ✅ feito 2026-11-23 |
| 8 | p08 | Header: busca com ícone + placeholder de atalho (⌘K) estilo premium; pill do governor com dot pulsante | visual consistente | ✅ feito 2026-11-23 |
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
- **2026-11-23 05:2x — loop (cron)**: p05 implementado — `.kpi .num` com gradiente
  sutil `--tx`→`--kc` via `background-clip:text`; `::after` com gradiente vertical
  fade nas pontas + glow `box-shadow` escalado por `--glow` (14px×0.6=8.4px
  confirmado) e hover 3px→4px com transição em `--micro`. Smoke 58/58 (100% OK);
  verificação visual no navegador embutido (screenshot + computed styles).
- **2026-11-23 06:3x — loop (cron)**: p06 implementado — drawer sai de
  `transform`-only para slide+fade em `--enter` com `visibility` atrasada via
  `transition-delay` (fecha suave, some após a transição); `.scrim` ganha
  `backdrop-filter: blur(0→8px)` progressivo em `--enter`; `.modal` deixa
  `display:none` e vira overlay sempre montado (opacity+visibility+pointer-events)
  com backdrop `var(--scrim)` + `blur(0→6px)` e `.box` com fade+slide+scale em
  `--enter` — entrada E saída animadas sem tocar no JS; `.palette .pbox` retimado
  para `--enter`. Smoke 58/58 (100% OK); computed styles no navegador embutido:
  `--enter`=400ms, estados finais opacity 1 / transform none / blurs aplicados.
- **2026-11-23 07:2x — loop (cron)**: p07 implementado — `.chip.on` sobe de
  acento 16%→20% no bg, ganha anel inset de acento 30% + glow externo
  `0 0 calc(12px*--glow)` a 25%, e o contador `.n` fica em acento com
  opacidade .95; `button.chip` ganha hover (borda acento 45% + texto `--tx`);
  transição do chip retimada de `all` para propriedades explícitas em
  `--micro`/`--ease`. Bug evitado: `box-shadow: var(--shadow-b), …` invalida a
  lista inteira quando `--shadow-b:none` (tema escuro) — substituído por anel
  inset. Smoke 58/58 (100% OK); computed styles + screenshot no navegador
  embutido (bg = mix exato 20% `--ac`×`--ov`, borda = `--ac`, anel+glow ok).
- **2026-11-23 08:2x — loop (cron)**: p08 implementado — `.searchbtn` premium:
  hover com borda acento 55% + anel inset 25% + glow `0 0 calc(14px*--glow)`
  (sem combinar com `--shadow-b`, que é `none` no escuro), ícone fica `--ac` no
  hover, `:active` com `scale(.985)`; `kbd ⌘K` vira keycap físico (gradiente
  `--sf`→`--bg`, luz superior inset 9%, sombra 1px 2px, tracking .05em).
  `.gov-pill` ganha `--dc` (`--ok`→`--wa` em `.warn`) e dot "radar": glow
  escalado por `--glow` + anel `::after` `govping` (scale .55→1.7 + fade) em
  `calc(2.2s*--motion)` — desliga com `--motion:0`/reduced-motion. Smoke 58/58
  (100% OK); computed styles + screenshot no navegador embutido (ring animando,
  dot âmbar em warn 508/600 MB; kbd oculto <900px pela regra mobile já
  existente, estilos confirmados via computed).
