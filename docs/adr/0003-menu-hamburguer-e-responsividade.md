# ADR 0003 — Menu hambúrguer e responsividade

- Status: Aceito
- Data: 2026-11-20

## Contexto

Requisito: menus hambúrguer que abram e fechem, responsividade máxima. O HTML
base usava barra de navegação fixa lateral que virava bottom-bar no mobile —
sem botão de menu.

## Decisão

- **≤ 1180 px**: a `<nav>` vira drawer off-canvas (esquerda), aberta pelo botão
  hambúrguer no header; fecha por scrim, tecla Esc, botão "Fechar" ou ao trocar
  de tela. Transição com física de mola `cubic-bezier(.3,.9,.32,1.06)`.
- **≤ 900 px**: idem + Estúdio vira bottom-sheet (botão "Estúdio" no header);
  dots de tema e botões secundários do header são ocultados.
- O scrim é compartilhado entre drawer lateral, drawer de detalhe e bottom-sheet
  do Estúdio — cada função de fechamento sabe recalcular se o scrim deve cair.

## Bug encontrado e corrigido (901–1180 px)

Sintoma: drawer aberto exibia só ícones, sem rótulos. Causa: o bloco do novo
breakpoint não anulava as regras do breakpoint antigo (`display:none` nos rótulos
e `justify-content:center`). Correção: regras `display:revert`,
`justify-content:flex-start` e reposicionamento da badge dentro do drawer.

## Consequências

- Nenhuma dependência de breakpoint JS; tudo CSS puro + classes `open`.
- Acessibilidade: `aria-expanded` sincronizado no hambúrguer e no botão Estúdio.
