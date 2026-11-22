# Security Analysis

**Há uma oportunidade de produto, mas ela é mais interessante como “plataforma de homologação de software de terceiros” do que como “scanner local com IA”.**

O cliente não compra apenas a identificação de uma vulnerabilidade. Ele compra uma decisão operacional defensável:

> “Podemos permitir que esta versão deste software entre no nosso ambiente? Quais evidências sustentam essa decisão e o que ficou sem testar?”

A arquitetura deve separar a execução não confiável da gestão e da decisão:

```visualizer
<svg width="100%" viewBox="0 0 680 440" xmlns="http://www.w3.org/2000/svg">
<defs><marker id="a" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0 0L8 4L0 8" fill="var(--color-text-secondary)"/></marker></defs>
<g font-family="var(--font-sans)" fill="var(--color-text-primary)">
<g class="c-blue">
<rect x="20" y="20" width="640" height="76" rx="12" stroke-width=".5"/>
<text class="th" x="40" y="47" font-size="16">Plano de controle — sem executar o projeto recebido</text>
<text class="ts" x="40" y="72">Políticas, fila, identidades, atualizações verificadas e aprovação humana</text>
</g>
<path d="M340 98V135" stroke="var(--color-text-secondary)" fill="none" marker-end="url(#a)"/>
<rect x="20" y="146" width="640" height="163" rx="12" fill="none" stroke="var(--color-text-warning)" stroke-dasharray="5 4"/>
<text x="40" y="170" font-size="13" fill="var(--color-text-warning)">ZONA DE EXECUÇÃO NÃO CONFIÁVEL — máquinas virtuais descartáveis</text>
<g class="c-amber">
<rect x="40" y="193" width="175" height="88" rx="8" stroke-width=".5"/>
<text class="th" x="127" y="221" text-anchor="middle" font-size="14">Instalação e build</text>
<text class="ts" x="127" y="248" text-anchor="middle">Captura dos artefatos</text>
<rect x="252" y="193" width="175" height="88" rx="8" stroke-width=".5"/>
<text class="th" x="339" y="221" text-anchor="middle" font-size="14">Execução e testes</text>
<text class="ts" x="339" y="248" text-anchor="middle">Processos, disco e rede</text>
<rect x="464" y="193" width="175" height="88" rx="8" stroke-width=".5"/>
<text class="th" x="551" y="221" text-anchor="middle" font-size="14">Análise isolada</text>
<text class="ts" x="551" y="248" text-anchor="middle">Binários e evidências</text>
</g>
<path d="M218 236H242M430 236H454" stroke="var(--color-text-secondary)" fill="none" marker-end="url(#a)"/>
<path d="M551 312V347" stroke="var(--color-text-secondary)" fill="none" marker-end="url(#a)"/>
<g class="c-blue">
<rect x="20" y="358" width="640" height="62" rx="12" stroke-width=".5"/>
<text class="th" x="40" y="382" font-size="14">Evidências validadas → avaliação por política → decisão com validade limitada</text>
<text class="ts" x="40" y="404">Setas representam transferências controladas; não garantem isolamento absoluto.</text>
</g>
</g>
</svg>
```

Abaixo apresento um plano técnico e de negócios. **Os segmentos, a sequência de desenvolvimento e as propostas comerciais são hipóteses estratégicas, não resultados de uma pesquisa de mercado ou de entrevistas com clientes.** As referências consultadas servem para contextualizar práticas técnicas e ofertas existentes.

### 1. Definição do produto: qual problema ele resolveria?

Sua proposta combina necessidades que normalmente aparecem separadas:

| Necessidade | Pergunta do cliente | Entrega do produto |
|---|---|---|
| Risco de dependências | “Este software contém componentes vulneráveis?” | Inventário, SBOM e correlação com vulnerabilidades conhecidas |
| Comportamento suspeito | “O que acontece quando ele é instalado e executado?” | Linha do tempo de processos, arquivos e comunicações |
| Procedência | “De onde veio este binário?” | Identidade do artefato, origem e evidências de build |
| Mudança de versão | “O que mudou desde a versão aprovada?” | Comparação estática e comportamental |
| Governança | “Quem autorizou seu uso e em quais condições?” | Aprovação, exceções, justificativas e histórico |
| Confidencialidade | “Posso fazer tudo sem enviar o software para terceiros?” | Execução e análise locais, com modos de operação definidos |

**O produto não precisa substituir SAST, antivírus, EDR ou gestão de dependências.** Ele pode coordenar essas análises e transformar os resultados em um processo de admissão de software.

O diferencial mais promissor seria:

> **Homologação local de artefatos executáveis, com instalação e execução observadas, comparação entre versões e evidências auditáveis.**

### 2. A evolução mais valiosa: de fotografia para histórico de comportamento

Uma análise isolada produz uma fotografia. Um produto recorrente precisa acompanhar mudanças.

#### Evolução A — Comparação comportamental entre versões

Em vez de analisar cada versão sem contexto:

- Execute a versão anteriormente aprovada e a candidata.
- Use o mesmo ambiente e os mesmos cenários.
- Compare processos, arquivos, bibliotecas e destinos de rede.
- Destaque diferenças relevantes.

**Exemplo hipotético:** uma ferramenta que antes apenas processava arquivos passa a executar um shell durante a instalação e a tentar acessar um novo domínio.

Isso não prova malícia, mas oferece uma pergunta objetiva para revisão.

**Valor comercial:** a segurança deixa de revisar tudo novamente e passa a revisar mudanças.

**Dificuldade técnica:** comportamentos legítimos também variam por horário, identificadores aleatórios, cache e serviços externos. Será necessário normalizar diferenças irrelevantes sem esconder mudanças perigosas.

#### Evolução B — Políticas por finalidade de uso

O mesmo comportamento pode ser aceitável em um software e inadequado em outro.

Uma ferramenta de backup precisa ler muitos arquivos. Um conversor simples talvez não devesse acessar diretórios de credenciais.

O produto pode permitir declarar:

- O que o software deve fazer.
- Quais diretórios pode acessar.
- Quais serviços externos são esperados.
- Se pode criar subprocessos.
- Se pode instalar serviços ou mecanismos de inicialização.
- Quais privilégios são necessários.

**A comparação deixa de ser apenas “parece malware?” e passa a ser “condiz com a função autorizada?”.**

Essas políticas devem ser propostas e revisadas, não inferidas automaticamente e aceitas como verdade.

#### Evolução C — Relacionar fonte, build e binário publicado

Para projetos open source, uma evolução importante é analisar separadamente:

1. O código obtido.
2. O resultado da compilação local.
3. O binário publicado pelo mantenedor.

Quando houver suporte a builds reproduzíveis, compare os resultados. Quando não houver, compare componentes e estruturas relevantes, registrando as limitações.

**Divergência não comprova comprometimento:** diferenças de compilador, timestamps, assinatura e ambiente podem explicar o resultado.

**Valor comercial:** investigar se o artefato distribuído corresponde ao que se espera do projeto.

#### Evolução D — Reavaliação contínua sem executar tudo novamente

Preserve o inventário e a identidade dos artefatos. Quando houver novas informações:

- Reavalie os componentes contra bases atualizadas.
- Reaplique políticas.
- Reabra aprovações afetadas.
- Solicite nova execução apenas quando necessário.

É importante distinguir:

- **Reavaliação de metadados:** nova informação sobre um artefato já conhecido.
- **Nova análise dinâmica:** execução de cenários adicionais.
- **Novo artefato:** outro hash, exigindo sua própria avaliação.

#### Evolução E — Evidências verificáveis para a cadeia de entrega

Uma aprovação pode gerar uma declaração assinada contendo:

- Hash do artefato.
- Ambiente e ferramentas utilizados.
- Cenários realizados.
- Versões das regras e bases.
- Limitações.
- Decisão, responsável e condições de validade.

**A assinatura comprova a origem e a integridade da declaração, não a segurança do software.**

No futuro, um repositório interno ou pipeline poderia verificar essa declaração antes de permitir a distribuição do artefato.

### 3. Viabilidade técnica: o que é fácil, difícil e arriscado?

**Minha avaliação é que o produto é tecnicamente viável com escopo estreito. A promessa de analisar automaticamente qualquer projeto, em qualquer sistema, com cobertura elevada, não é um MVP realista.**

| Capacidade | Avaliação | Principal dificuldade |
|---|---|---|
| Hashes, inventário e análise estática básica | Viável para o início | Formatos malformados e identificação incompleta |
| SBOM e CVEs | Viável com ferramentas existentes | Bibliotecas embutidas, versões e metadados ausentes |
| Execução de uma CLI Linux conhecida | Viável com perfil de execução | Preparação de entradas e ambiente |
| Comparação de disco e árvore de processos | Viável com limitações | Atribuição de eventos e ruído |
| Instalar e executar qualquer repositório | Complexidade alta | Dependências, documentação e requisitos heterogêneos |
| Visibilidade independente do guest | Complexidade alta | Integração com hypervisor e interpretação dos eventos |
| Análise de Windows, Linux e macOS | Expansão relevante de escopo | Instrumentação, distribuição e licenciamento |
| Detectar malware evasivo de forma abrangente | Pesquisa contínua | Evasão, gatilhos e cobertura de cenários |
| Descobrir vulnerabilidades inéditas automaticamente | Incerto e especializado | Validação, exploração de estados e falsos positivos |

#### O maior gargalo pode ser executar corretamente o software

Um scanner pode terminar sem alertas porque a aplicação sequer inicializou de forma útil.

Por isso, o produto deve registrar estados distintos:

- Preparação concluída ou falhou.
- Build concluído ou falhou.
- Aplicação iniciou ou não.
- Cenários foram realizados ou não.
- Telemetria completa ou parcial.
- Achados presentes ou ausentes.
- Resultado conclusivo para a política ou inconclusivo.

**“Não detectado” nunca pode significar automaticamente “aprovado”.**

#### Dois perfis de execução são melhores do que um

Há uma tensão entre endurecimento e fidelidade:

- Um ambiente muito restritivo pode impedir a funcionalidade legítima.
- Um ambiente permissivo pode observar mais comportamento, mas eleva o risco operacional.

Uma evolução adequada é oferecer:

**Perfil restritivo:** menor superfície de ataque, sem rede externa, privilégios mínimos.

**Perfil representativo:** permissões necessárias à função, dados sintéticos e serviços simulados, ainda dentro de isolamento dedicado.

A comparação entre os perfis ajuda a distinguir falha por restrição de comportamento realmente inesperado.

#### “Local”, “sem internet” e “air-gapped” não são sinônimos

O produto deve declarar precisamente seus modos:

- **Local:** processamento no ambiente do cliente.
- **Sem saída externa durante o teste:** execução sem acesso à internet, mas infraestrutura possivelmente conectada.
- **Rede isolada de laboratório:** serviços internos simulados.
- **Air-gapped:** separação operacional, com importação e exportação controladas.

Oferecer operação air-gapped exige mais do que desligar a rede: atualização offline, licenciamento offline, documentação, diagnóstico e suporte compatíveis.

### 4. Arquitetura de produto: onde investir engenharia própria

Não recomendo criar um novo antivírus ou um novo banco de CVEs no início.

#### Componentes a integrar

- Ferramentas de inventário e análise de componentes.
- Regras e assinaturas de análise estática.
- Hypervisor e mecanismos de snapshots.
- Instrumentação de processos, arquivos e rede.
- Formatos de relatório e integração existentes.

A [OWASP reúne práticas de endurecimento e exemplos de scanners e monitoramento comportamental](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html). Ela também reforça que containers compartilham o kernel e que rootless não elimina essa fronteira de risco.

#### Componentes nos quais construir valor próprio

**1. Orquestração segura**

Criar, executar, encerrar e destruir ambientes de modo verificável, inclusive quando o job falha.

**2. Perfis de execução**

Descrever entradas, comandos, serviços simulados e critérios de sucesso de um teste.

**3. Modelo unificado de evidências**

Relacionar artefato, cenário, processo, evento e achado sem perder a origem de cada informação.

**4. Comparação entre versões**

Reduzir ruído e destacar mudanças explicáveis.

**5. Motor de políticas**

Separar evidência observada, regra aplicada e decisão humana.

**6. Fluxo de aprovação**

Exceções com justificativa, prazo, responsável e possibilidade de revisão.

#### A fronteira de saída merece tanta atenção quanto a entrada

Um projeto malicioso pode produzir:

- Logs enormes.
- Arquivos comprimidos com expansão excessiva.
- HTML com scripts.
- Nomes e mensagens com sequências de controle.
- Conteúdo destinado a manipular uma IA.
- Arquivos que exploram parsers dos analisadores.

Portanto, o plano de controle não deve abrir indiscriminadamente resultados produzidos pelos workers. São necessários limites, validação de formatos, renderização segura e análise intermediária.

**Assinar uma evidência depois de coletá-la também não corrige uma coleta comprometida.** O produto precisa informar a origem da telemetria e seu grau de resistência à adulteração.

### 5. MVP recomendado

Eu começaria com **artefatos Linux e aplicações de linha de comando**, em infraestrutura dedicada do cliente.

#### Dentro do MVP

- Recebimento de binário ou pacote com instruções explícitas de instalação.
- Identificação por hash.
- Análise estática e inventário.
- Execução em VM descartável.
- Cenário definido pelo operador.
- Registro de processos, arquivos e tentativas de rede.
- Comparação de estado antes e depois.
- Relatório com evidências e limitações.
- Aprovação manual e histórico.
- Comparação com a versão anterior, inicialmente simples.

#### Fora do MVP

- Executar qualquer repositório sem configuração.
- Suporte simultâneo a todos os sistemas operacionais.
- Instalação de drivers e análise de kernel.
- Testes automáticos de exploração.
- Descoberta geral de vulnerabilidades inéditas.
- Aprovação autônoma por IA.
- Exposição do laboratório à internet para “ver o que acontece”.
- SaaS multitenant para amostras de alto risco.

**O MVP precisa provar que a evidência muda uma decisão, não apenas que consegue iniciar uma VM e gerar um relatório.**

### 6. Quem compraria primeiro?

A seleção do público deve considerar dor, frequência, orçamento e dificuldade de implantação.

| Segmento candidato | Caso de uso | Comprador provável | Obstáculo |
|---|---|---|---|
| Times de AppSec e segurança de plataforma | Aprovar novas ferramentas e atualizações | Liderança de segurança ou engenharia | Integração ao processo existente |
| Organizações com restrição de envio à nuvem | Analisar internamente software de terceiros | Segurança e infraestrutura | Exigências de implantação e suporte |
| Consultorias de segurança | Padronizar avaliações para clientes | Liderança técnica | Necessidade de personalização |
| Equipes de resposta a incidentes | Investigar artefatos suspeitos | SOC ou resposta a incidentes | Expectativa de análise especializada |
| Desenvolvedores individuais | Avaliar projetos antes de instalar | Próprio usuário | Disposição a pagar ainda não validada |

**Minha hipótese de entrada seria um time de segurança de plataforma que já possui uma fila recorrente de homologação.**

É um problema operacional identificável: existem solicitações, responsáveis, tempo gasto e decisões a documentar. Isso facilita medir valor.

O posicionamento inicial não deveria misturar esse caso com um laboratório completo de malware para pesquisadores. Os requisitos de profundidade e evasão são diferentes.

### 7. Concorrência: “scanner local com IA” já não é diferenciação suficiente

Nas páginas consultadas:

- O [Kekkai](https://kademos.org/kekkai) se apresenta como um orquestrador de ferramentas existentes, com normalização, triagem e integração ao fluxo de segurança.
- O [Thresher](https://thresher.sh/) anuncia análise local em VM, múltiplos scanners e analistas de IA.

**São descrições dos próprios projetos, não resultados de uma avaliação independente realizada aqui.** Também não representam um levantamento completo da concorrência.

A implicação estratégica é clara: **agregar scanners e acrescentar um resumo por IA é uma proposta fácil de comparar e potencialmente fácil de substituir.**

Diferenciais mais defensáveis seriam:

- Execução reproduzível por cenário.
- Comparação comportamental confiável.
- Operação offline realmente suportada.
- Evidência rastreável e com cobertura explícita.
- Integração à admissão de software.
- Redução comprovada do trabalho de revisão.
- Qualidade e manutenção dos perfis de execução.

A defesa competitiva pode estar mais no fluxo operacional e na qualidade dos dados do que em um modelo de IA proprietário.

### 8. Modelo de negócios recomendado

#### Etapa inicial — Serviço produtizado

Ofereça uma avaliação com escopo fechado:

- Tipo e quantidade de artefatos acordados.
- Cenários definidos.
- Ambiente de execução especificado.
- Evidências entregues.
- Reunião de triagem.
- Limitações e critérios de aceite.

Isso permite descobrir o que realmente exige intervenção humana.

**Risco:** virar uma consultoria que desenvolve um laboratório diferente para cada cliente.

**Mitigação:** catálogo de formatos suportados, perfis reutilizáveis e limites claros de customização.

#### Etapa seguinte — Plataforma on-premises com assinatura

Modelo possível:

- Licença da plataforma.
- Capacidade de execução contratada.
- Atualizações de regras e perfis.
- Suporte.
- Implantação cobrada separadamente.
- Revisão especializada opcional.

Uma unidade comercial razoável pode ser **concorrência de execução**, como capacidade de workers, combinada com limites de uso e armazenamento.

Isso tende a refletir melhor o custo do que cobrar apenas por usuário: poucos operadores podem gerar carga muito elevada.

Evitaria cobrança por quantidade de vulnerabilidades encontradas. Ela cria incentivos ruins e torna o gasto imprevisível.

#### Expansão — Integração embarcada

Depois de estabilizar o motor, ofereça integração para:

- Repositórios internos de artefatos.
- Portais de software corporativo.
- Pipelines de entrega.
- Plataformas de segurança.
- Consultorias que precisam incorporar análise a seus serviços.

**Vantagem:** entrar no fluxo que já existe.

**Risco:** perder controle da experiência e assumir compromissos de compatibilidade e disponibilidade cedo demais.

### 9. Viabilidade econômica: como avaliar sem inventar receita

Não há dados suficientes para estimar faturamento, margem ou ponto de equilíbrio. O primeiro modelo financeiro deve ser alimentado por pilotos.

#### Custos que precisam ser medidos

**Por execução:**

- Tempo de CPU e memória.
- Armazenamento temporário e retenção.
- Inicialização e descarte do ambiente.
- Processamento dos scanners.
- Inferência local, se houver.
- Reexecuções e falhas.
- Minutos de revisão humana.

**Por cliente:**

- Implantação.
- Integrações.
- Atualizações.
- Suporte.
- Manutenção de perfis específicos.
- Atendimento a requisitos de segurança e compras.

**Custos fixos:**

- Engenharia.
- Pesquisa e validação.
- Infraestrutura de testes.
- Gestão das dependências do produto.
- Comercial e jurídico.

#### Duas contas simples orientam a decisão

A margem de contribuição por cliente precisa considerar os serviços que parecem “gratuitos”:

$$
MC = R - C_{\mathrm{execucao}} - C_{\mathrm{suporte}} - C_{\mathrm{revisao}} - C_{\mathrm{outros\ variaveis}}
$$

Em que a receita e os custos devem usar o mesmo período.

E o ponto de equilíbrio operacional pode ser aproximado por:

$$
N_{\mathrm{equilibrio}} =
\left\lceil
\frac{C_{\mathrm{fixo}}}{MC_{\mathrm{medio\ por\ cliente}}}
\right\rceil
$$

Essa simplificação só é útil com margem positiva e clientes relativamente comparáveis.

**O principal risco econômico é humano:** cada software exigir investigação artesanal para instalar, funcionar e explicar os resultados.

Por isso, o piloto deve medir especialmente:

> **Quantos minutos de especialista são necessários para produzir uma avaliação utilizável?**

O fato de o processamento ocorrer no hardware do cliente não elimina seus custos de produto nem os custos operacionais dele.

### 10. Como provar valor ao cliente

Evite justificar a compra com uma estimativa arbitrária de “incidentes evitados”.

Comece por métricas observáveis:

| Dimensão | Métrica |
|---|---|
| Produtividade | Tempo de homologação antes e depois |
| Automação | Parcela das avaliações concluídas sem intervenção técnica |
| Qualidade | Achados relevantes confirmados por revisão |
| Ruído | Alertas descartados e tempo de triagem |
| Cobertura | Cenários efetivamente executados e telemetria disponível |
| Operação | Falhas de execução, reprocessamentos e descarte confirmado |
| Negócio | Reutilização, renovação e expansão do piloto |
| Decisão | Avaliações que alteraram aprovação, exceção ou remediação |

Para avaliar detecção, use um conjunto controlado com comportamentos conhecidos, amostras benignas e casos não utilizados para ajustar as regras.

**Não transforme desempenho nesse conjunto em promessa de detecção universal.**

### 11. Abordagem comercial: vender a decisão, não a ferramenta

#### Mensagem principal

> “Ajudamos sua equipe a homologar software de terceiros com execução isolada, evidências de comportamento e comparação entre versões, sem retirar os artefatos do seu ambiente.”

#### Perguntas de descoberta

- Como uma ferramenta nova é aprovada hoje?
- Quem autoriza a atualização de uma ferramenta já aprovada?
- Quais evidências são exigidas?
- Quanto trabalho é manual?
- Quais softwares não podem ser enviados à nuvem?
- O que acontece quando uma avaliação é inconclusiva?
- Quem tem orçamento para resolver o problema?
- Que resultado justificaria um piloto pago?

#### Estrutura de um piloto comercial

**Entrada:** conjunto representativo de artefatos e versões, com autorização e cenários acordados.

**Execução:** avaliação lado a lado com o processo atual.

**Saída:** relatório comparativo, decisões que mudaram, esforço necessário e limitações.

**Aceite:** critérios definidos antes do teste, incluindo operação, qualidade de evidência e esforço humano.

**Conversão:** implantação recorrente somente se houver uso repetido e benefício mensurável.

A aprovação do piloto deve envolver tanto quem usa o produto quanto quem autoriza sua infraestrutura. Um laboratório de execução de código não confiável precisa passar pela própria avaliação de risco do cliente.

### 12. Roadmap orientado por critérios, não por promessa de calendário

| Fase | Entrega | Condição para avançar |
|---|---|---|
| Descoberta | Caso de uso, comprador e fluxo atual documentados | Dor recorrente e disposição a testar com compromisso real |
| Prova técnica | Uma classe de artefato executada com coleta e descarte | Evidências úteis e isolamento validado dentro do modelo de ameaça |
| Piloto assistido | Processo repetível com revisão humana | Benefício frente ao processo atual e custo de atendimento conhecido |
| Produto inicial | Implantação padronizada, políticas e histórico | Uso recorrente sem customização excessiva |
| Diferenciação | Comparação entre versões e perfis reutilizáveis | Redução demonstrável da revisão manual |
| Escala | Mais formatos, integrações e operação offline avançada | Demanda contratual e capacidade de suporte sustentável |

Eu adiaria expansão de sistemas operacionais e autonomia de IA até o fluxo principal funcionar de forma repetível.

### 13. Riscos de produto e como tratá-los

#### Falsa sensação de segurança

**Tratamento:** decisões condicionais, cobertura explícita, estado inconclusivo e validade limitada.

#### Comprometimento da própria plataforma

**Tratamento:** separar planos, reduzir interfaces, atualizar componentes, validar entradas e saídas e submeter o produto a avaliação independente.

#### Dependência excessiva de ferramentas externas

**Tratamento:** adaptadores, versões controladas, testes de regressão e revisão das licenças de ferramentas, regras e bases.

#### Dados sensíveis em evidências

Memória, logs e capturas podem conter credenciais ou dados pessoais, mesmo em testes.

**Tratamento:** dados sintéticos, minimização, retenção configurável, criptografia e controle de acesso. No Brasil, a avaliação contratual e de privacidade deve considerar a LGPD conforme os dados e o papel de cada parte; isso exige análise específica, não uma alegação genérica de conformidade.

#### Uso indevido

**Tratamento:** escopo autorizado, controle de usuários e ausência de automação ofensiva fora do laboratório.

#### Confusão entre atestado e certificação

**Tratamento:** chamar a entrega de relatório ou declaração de avaliação. Não vender “certificação de software seguro” sem base normativa e competência aplicável.

### 14. Critérios para continuar, reduzir ou interromper a iniciativa

**Sinais favoráveis:**

- O cliente já faz homologação manual.
- Existe frequência suficiente de novas versões.
- As evidências mudam decisões.
- Os perfis podem ser reutilizados.
- O cliente aceita pagar pelo fluxo, não apenas por uma demonstração.
- A necessidade de manter os artefatos localmente é real.

**Sinais de alerta:**

- Cada avaliação vira um projeto de consultoria.
- A aplicação raramente consegue executar cenários úteis.
- Os relatórios repetem informações que o cliente já possui.
- Há interesse técnico, mas nenhum responsável por orçamento.
- O produto depende de prometer “detecção de tudo”.
- O custo de implantação e suporte supera a receita provável.

Se houver valor apenas na avaliação especializada, pode existir um bom negócio de serviço — mas isso não prova que haverá uma plataforma escalável.

### Recomendação final

**Eu avançaria, mas com uma proposta deliberadamente estreita: homologação local e recorrente de ferramentas Linux de terceiros, com foco em mudanças de versão e evidências operacionais.**

A sequência recomendada é:

1. Validar uma fila real de homologação.
2. Entregar avaliações assistidas e pagas.
3. Medir esforço humano e reutilização.
4. Automatizar o que se repete.
5. Integrar aprovações ao repositório interno de software.
6. Expandir formatos e profundidade de análise somente com demanda comprovada.

O melhor ativo a construir não é “uma IA que vasculha tudo”. É **um processo confiável para responder o que foi executado, o que foi observado, o que mudou e por que uma organização decidiu aceitar ou rejeitar aquele artefato**.

Quer que eu estruture um plano de validação de negócio de 90 dias, com backlog do MVP, critérios dos pilotos e modelo de proposta comercial?