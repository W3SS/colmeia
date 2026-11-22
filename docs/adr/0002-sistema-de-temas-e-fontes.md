# ADR 0002 — Sistema de temas (4 base + retrô + energia + 18 cyberpunk) com fontes combinadas

- Status: Aceito
- Data: 2026-11-20

## Contexto

O contrato exige os 4 temas base (claro, escuro, sépia, neumorph). O produto
pediu além: opções de tema das combinações das imagens de referência (OS retrô,
paletas de energia/sponsor, cyberpunk) e **pelo menos 15 temas cyberpunk**, com
fontes que combinem com cada tema — sem pesar o software.

## Decisão

- Catálogo **data-driven** (`THEME_DEFS` no próprio HTML): cada tema define os
  tokens completos (`--bg/--sf/--ov/--bd/--tx/--mu`, status, 8 cores de dados,
  sombras neumorph opcionais, raio, largura de borda, `color-scheme`).
- `applyThemeVars()` injeta os tokens inline no `<html>`, sobrescrevendo
  qualquer tema; os 4 temas base existem também como CSS estático para
  first-paint sem JS.
- Grupos: **Base (4)**, **Retrô (6)**, **Energia (5)**, **Cyberpunk (18)** — 33 no total.
- Fontes: stacks por tema via `--font-display/--font-body/--font-mono`
  (Orbitron, Audiowide, Oxanium, Bruno Ace SC, Share Tech Mono; retrô usam
  mono/sistema). Google Fonts carregado com truque `media="print" onload` —
  **progressive enhancement**: offline, cai para fontes do sistema.
- Temas cyberpunk ativam `data-cyber="1"`: brilho neon nos títulos, sheen
  animado nos cards de tema e realce de bordas por acento.

## Alternativas consideradas

- **Baixar e embedar woff2**: descartada — cada família ~100–300 KB quebraria o
  orçamento de 200 MB do contrato e do primeiro paint.
- **CSS-in-JS / classe por tema**: descartada — o catálogo em objeto JS é menor
  e permite ações em massa (paleta ⌘K, galeria do Estúdio).

## Consequências

- Adicionar um tema = uma entrada em `THEME_DEFS` (uma linha de tokens).
- A galeria do Estúdio e a paleta de comandos são geradas do catálogo.
- Contraste extra (`data-contrast`) reaplicado após injeção dos tokens.
