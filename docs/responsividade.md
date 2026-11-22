# Análise de Responsividade — Web GUI Colmeia

> Data: 2026-11-21 (atualização — nav/estúdio como overlays) · Arquivo analisado:
> `web/index.html` (CSS embutido, arquivo único)
> Método: auditoria por inspeção de CSS + verificação empírica no browser

## 1. Arquitetura de layout (atual)

Desde o redesign do menu (rail escuro + painel claro, referência do usuário),
a aplicação **não usa mais colunas fixas**: o `<main>` ocupa toda a largura
(`.app{grid-template-columns:1fr;grid-template-areas:"main"}`) com
`padding-left:74px` para o trilho persistente.

- **Trilho (`.rail`) SEMPRE visível** em todos os breakpoints: cartão escuro
  de 54 px com logo hexagonal + ícones brancos das telas, clicável como
  navegação direta (menu "fechado" = só ícones, conforme a referência).
- **Painel (`.npanel`) expande do próprio trilho** ao clicar no hambúrguer
  (`nav.open .npanel`, deslize de mola 300 ms · `--motion`) — sem criar barra
  nova. Fechado, o painel fica totalmente fora da tela
  (`translateX(calc(-100% - 62px))` + `nav{width:62px;overflow:hidden}`), sem
  interceptar cliques no trilho.
- **Estúdio = painel fixo no canto superior direito** (`position:fixed`,
  `top:56px;right:0;bottom:0`, 336 px, z-index 24), aberto pelo botão
  "Estúdio" — último item do header, depois de "＋ Novo projeto".
- **nav e estúdio ficam FORA do `.app` no DOM** (bug do scrim: dentro do
  contexto de empilhamento z-index:1 do `.app`, o scrim z-index:19 da raiz
  cobria todos os cliques). Header em z-index:26, sempre clicável.
- Scrim único (`data-act="close-drawer"`) fecha painel, drawer e estúdio; Esc
  também fecha os três.

## 2. Breakpoints reais

### Todos os tamanhos — nav drawer + estúdio overlay

- Hambúrguer (`#burger`) sempre visível no canto superior esquerdo;
  botão "Estúdio" (`.studio-btn`) sempre visível no canto superior direito,
  oposto ao hambúrguer.
- A `< 900px`: `.hdr-right .btn:not(.studio-btn){display:none}` esconde os
  demais botões do header mantendo o Estúdio; margens do drawer reduzidas
  (`rail` 6 px, painel `min(210px,58vw)`).

### ≤ 1180 px

- `.cols-2`, `.cols-3`, `.panes3` colapsam para 1 coluna.

### ≤ 900 px

- Trilho mantido (nav `left:6px;top:60px`, painel `min(205px,56vw)`);
  `.app{padding-left:66px}`.
- `.kpis` vira 2 colunas; `.tgrid` cards mínimos de 160 px; theme-dots,
  kbd da busca e texto do gov-pill ocultos; botões do header exceto o
  Estúdio ocultos; drawer de detalhe full-width (GitFlow vira 100 vw).

## 3. Verificação empírica (2026-11-21)

- Screenshot desktop com menu aberto: trilho escuro + painel com seções,
  contagens e badges fiéis à referência; item ativo destacado.
- Estado padrão pós-boot: trilho sempre visível e clicável
  (`elementFromPoint` acerta o botão do rail); painel totalmente fora da
  tela (borda direita medida em x=10); com o menu aberto, `elementFromPoint`
  sobre um item acerta o item (não o scrim — bug corrigido).
- Estúdio aberto: painel ancorado abaixo do header à direita, botão fechar no
  topo, galeria de temas rolável.
- Estúdio aberto: painel ancorado abaixo do header à direita, botão fechar no
  topo, galeria de temas rolável.
- Smoke test: 52/52 verificações (novas: drawer com rail+npanel, sem coluna
  fixa, estúdio fechado por padrão no canto superior direito).

## 4. Limitações conhecidas

- Canvas de branches com largura fixa 920 px rola horizontalmente em telas
  estreitas (sistema de coordenadas do canvas).
- No fechamento do painel há um salto discreto de `width` do `nav`
  (62 px → auto) — imperceptível porque o painel desliza por cima.
