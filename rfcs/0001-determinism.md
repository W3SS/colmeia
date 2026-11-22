# RFC 0001 — Contrato de Determinismo

* **Status:** Aprovado / Normativo
* **Autor:** Colmeia App Architecture Team
* **Data:** 2026-11-20

## 1. Princípios Fundamentais

O projeto **Colmeia** estabelece que o determinismo é um requisito não funcional de primeira classe, não uma conveniência secundária. Todas as operações automáticas sobre repositórios de código devem produzir efeitos colaterais reproduzíveis e verificáveis.

1. **Ordenações Estáveis com Tie-Break:**
   * Toda iteração sobre arquivos, diffs, commits, símbolos ou filas de jobs deve ser ordenada lexicograficamente estável.
   * Empates são desempatados obrigatoriamente por hash criptográfico de conteúdo (`SHA-256`), nunca pela ordem de arquivo no sistema de arquivos subjacente (`readdir`).

2. **Recipes Replayáveis:**
   * Qualquer operação de alteração (como `slice` ou resolução de `merge`) deve persistir uma *recipe* contendo:
     - `input_hashes`: SHA dos commits/blobs de entrada.
     - `policy_hash`: SHA-256 da política/regras aplicadas.
     - `output_hash`: SHA-256 resultante.
     - `plan_json`: Plano estruturado e serializado em JSON.
   * O comando `colmeia replay <recipe-id>` deve reproduzir o resultado idêntico bit a bit.

3. **Operações Atômicas de Referência (CAS):**
   * Modificações em branches ou refs locais utilizam exclusivamente compare-and-swap (`git update-ref <new> <old>`). Se o ref divergiu desde a última leitura, a operação aborta sem sobrescrever o histórico silenciosamente.

4. **Recuperação de Crash sem Corrupção:**
   * Fila de execução de jobs utiliza *leases* temporais (`lease_until`). Se um worker for terminado abruptamente (`kill -9`, falta de energia, estouro de memória), o job volta a ficar disponível para reexecução após a expiração do lease.
