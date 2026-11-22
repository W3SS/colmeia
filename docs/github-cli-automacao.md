# Automação GitHub CLI + Hugging Face no Colmeia

**Status:** implementado (Lote 10) — `colmeia ci-scan`
**Data:** 2026-11-22

Este documento registra as decisões de arquitetura para automatizar tarefas do
GitHub (detecção de falhas de CI, triagem, assistência de código) com o máximo
de determinismo possível, conforme solicitado: reduzir loops e tornar o
comportamento reproduzível. Baseia-se na documentação oficial da GitHub CLI,
na documentação de Webhooks do GitHub, na documentação de Inference Providers
da Hugging Face e em padrões observados em repositórios de automação de CI.

---

## 1. Mapeamento: GitHub CLI → feature Colmeia

| Comando `gh` (documentação oficial) | Uso no Colmeia | Implementação |
|---|---|---|
| `gh run list --repo R --status failure --limit N --json databaseId,workflowName,headBranch,conclusion,createdAt,url` | Detecção estruturada de falhas de CI | `ghci::GhCli::detect_failures` |
| `gh run view <id> -R repo --log-failed` | Log **apenas dos passos que falharam** (nunca o log inteiro) | `ghci::GhCli::failure_log` (trunca em 8 000 chars) |
| `gh run rerun <id> -R repo --failed` | Triagem de flaky vs. falha real — só via flag explícita `--rerun` | `ghci::GhCli::rerun_failed` |
| `gh run watch <id>` | (roadmap) acompanhamento de rerun | — |
| `gh api repos/O/R/actions/runs --paginate` | Escape hatch REST para consultas fora do escopo da CLI | disponível ao operador; não encapsulado |
| `gh pr checks <n>` ⚠️ | **Não usado com `--watch`**: a flag sai com código 0 mesmo quando há falhas | preferimos `--json` + parse |
| `gh workflow run wf.yml -f k=v` | (roadmap) disparo de workflows de remediação | — |
| `gh webhook forward` | (roadmap) desenvolvimento local de handlers event-driven | — |

Fontes: <https://docs.github.com/pt/github-cli/github-cli>,
<https://cli.github.com/manual/>.

### Decisão D1 — Por que encapsular a CLI em vez da REST API direta?

A CLI já resolve autenticação não-interativa (`GH_TOKEN`/`GITHUB_TOKEN`),
paginação e formatos `--json` estáveis. Encapsular via `std::process::Command`
(zero crates novas — contrato de leveza) mantém o binário pequeno e torna os
testes trivialmente determinísticos (binário `gh` falso + PATH isolado).
Quando a CLI não cobre o caso, `gh api` é o escape hatch documentado.

### Decisão D2 — Mutação nunca é implícita

Todas as chamadas do scanner são somente-leitura. A única operação de escrita
(`rerun --failed`) exige a flag `--rerun` explícita do operador. Isso elimina
a classe de bug "loop de automação que re-executa jobs indefinidamente".

---

## 2. Determinismo e anti-loop

### 2.1 Dedup idempotente por `run_id` (banco local)

Tabela `ci_runs(run_id INTEGER PRIMARY KEY, repo, workflow, conclusion,
seen_at)`. Um run de CI é reportado **no máximo uma vez**, mesmo que o scan
rode em loop:

```sql
INSERT OR IGNORE INTO ci_runs …
```

`ghci::ci_scan` filtra `ci_seen(run_id)` antes de reportar. Segunda passada
sobre o mesmo estado devolve zero achados novos — comportamento verificado em
teste (`ci_scan_e_idempotente_por_run_id`).

### 2.2 Orçamento de requisições (rate budget)

Reutiliza o mecanismo existente `store.rate_consume("github-cli", 60/hora)`.
Sem isso, um cron de scan vira bombardeio de API; com ele, o pior caso é um
número fixo e conhecido de chamadas por hora. Slugs excedentes são contados em
`report.rate_limited`, não falham silenciosamente.

### 2.3 Entrada ordenada e deduplicada

`ci_scan` ordena (`sort_unstable`) e deduplica os slugs antes de iterar —
mesma entrada, mesma sequência de chamadas, mesma saída.

### 2.4 Event-driven (webhooks) como camada seguinte

A documentação de Webhooks do GitHub define os eventos que tornam a detecção
**reativa** em vez de por polling:

| Evento | Quando dispara | Informação-chave |
|---|---|---|
| `workflow_run` (`completed`) | Run de workflow termina | `conclusion`, `run_number`, URL |
| `workflow_job` (`completed`) | Job individual termina | **nome do step que falhou** (necessário para diagnóstico fino) |
| `check_run` / `check_suite` | Checks terminam | `conclusion` agregada |

Regras de determinismo para a fase event-driven:

- **Header `X-GitHub-Delivery`** é um GUID único por entrega → chave de dedup;
  entregas repetidas (retries do GitHub) são descartadas.
- **Anti-recursão**: workflows disparados pela própria Colmeia devem ignorar
  eventos cujo `actor` é a conta da automação.
- O Colmeia **já possui** `src/webhooks.rs` (verificação HMAC-SHA256) e
  `src/hooks_out.rs` (entrega de hooks externos) — a fase event-driven se
  apoia nesses módulos, sem novas dependências.

Fonte: <https://docs.github.com/pt/webhooks>.

---

## 3. Hugging Face — assistência de código (opt-in)

### 3.1 Endpoint e contrato

Inference Providers expõe um endpoint OpenAI-compatível:

```
POST https://router.huggingface.co/v1/chat/completions
Authorization: Bearer $HF_TOKEN
```

Fonte: <https://huggingface.co/docs/huggingface_hub/models-inference>.

Configuração por ambiente (sem arquivo de config adicional):

- `HF_TOKEN` — ausente → recurso desativado, erro limpo;
- `HF_MODEL` — default `Qwen/Qwen2.5-Coder-32B-Instruct`.

### 3.2 Determinismo da geração

- `temperature: 0` + modelo pinado → mesma entrada, mesma saída;
- `max_tokens: 512` fixo → custo e latência previsíveis em hardware mínimo;
- prompt do sistema restrito a "1 parágrafo: causa provável + correção",
  limitando variância de formato.

### 3.3 Fronteira de responsabilidade

A IA **resume e sugere**; nunca edita código nem dispara ações. O pipeline é:

```
log falho (--log-failed, ≤8k chars) → HF (temp 0) → sumário exibido ao operador
```

A ação continua humana (ou via receitas determinísticas já existentes no
Colmeia: F17 starlark, F14 merge estruturado).

---

## 4. Padrões adotados de outros repositórios de automação de CI

Levantamento sobre repositórios de skills/padrões `gh-fix-ci` /
`github-ci-fix`:

1. **Extrair o trecho relevante do log** (20–50 linhas ao redor de
   `error|fail|traceback|assert`) em vez do log inteiro → adotado via
   `--log-failed` + truncamento em 8 000 chars;
2. **Flaky vs. real**: re-executar só os jobs falhos (`rerun --failed`) — se
   passa, era transitório; se falha igual, é real → adotado como `--rerun`
   explícito;
3. **Plano antes de agir**: sumarizar e propor antes de qualquer mutação →
   adotado (HF só explica; ação exige flag);
4. **Fronteiras claras**: a automação cobre só GitHub Actions; CI externo
   (Jenkins, GitLab) fica de fora do escopo deste módulo.

---

## 5. Interface do operador

```console
# scan de todos os repos cadastrados com remote github.com
colmeia ci-scan

# um repo específico
colmeia ci-scan --repo owner/repo

# com explicação das falhas via Hugging Face (requer HF_TOKEN)
colmeia ci-scan --explain

# triagem de flaky: re-executa só os jobs falhos dos runs NOVOS detectados
colmeia ci-scan --rerun
```

Saída: uma linha `[CI-FALHA]` por run novo, sumário opcional, e rodapé com
contagem `novo(s) / dedupado(s) / rate-limited`. Cada achado gera um evento
`ci_failure` no store (visível na TUI/Web e consumível por hooks).

---

## 6. Testes

`tests/ghci_test.rs` — 11 testes de integração:

- parse do JSON canônico de `gh run list` e flags exatas enviadas;
- truncamento de log e argumentos de `rerun --failed`;
- exit ≠ 0 → `GhciError::Exit` (com código e stderr);
- `gh` ausente → `GhciError::NotInstalled`;
- idempotência do scan por `run_id` (2ª passada vazia);
- emissão de evento `ci_failure` por achado;
- ordenação/dedup de slugs (2 chamadas de API para 3 slugs);
- cliente HF contra sink TCP local: formato OpenAI-compat, `temperature: 0`,
  `Authorization: Bearer`, `max_tokens: 512`.

Testes unitários em `src/ghci.rs`: extração de slug de remote https/ssh,
truncamento UTF-8.

## 7. Roadmap (não implementado neste lote)

1. `gh run watch` após `--rerun` para concluir a triagem flaky;
2. Handler `workflow_run`/`workflow_job` em `webhooks.rs` (dedup por
   `X-GitHub-Delivery`);
3. `gh workflow run` para disparar receitas de remediação com aprovação;
4. Sumários agregados por causa raiz (agrupamento determinístico por hash do
   trecho de erro normalizado).
