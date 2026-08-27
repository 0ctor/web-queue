# Planejamento: Painel de Chamadas (web-queue)

Documento canônico de planejamento do produto **web-queue** no ecossistema Octor.
**Status:** planejamento — sem implementação nesta fase.

---

## Contexto atual

| O que existe | O que é | Serve para painel de chamadas? |
|---|---|---|
| **web-indoor** (`indoor.octor.com.br`) | Signage digital — playlist de mídias na TV, pareamento por código de 6 dígitos | Parcialmente (só a **casca TV**) |
| **web-agenda** — lista de espera | Fila interna de "aguardando vaga de horário" | Não — conceito diferente |
| **web-agenda** — status 8 (`awaiting`) | "Paciente chegou na clínica" | Sim — **fonte de dados** |
| **web-admin TvDashboard** | KPIs agregados para CS | Não — não é display na recepção |
| FAQ comercial (web-promoters) | Indoor **não é** totem nem painel de senhas hospitalar | Delimitação explícita de produto |

**Conclusão:** painel de chamadas é **greenfield**. Não há código de totem, senha, fila em tempo real nem display público de pacientes.

---

## As três opções avaliadas

### Opção A — Estender o web-indoor

Colocar "modo painel de chamadas" dentro do indoor, reutilizando player TV + código de pareamento.

| Prós | Contras |
|---|---|
| Reaproveita player fullscreen e pareamento | Mistura **marketing** (mídia estática) com **operação clínica** (tempo real, LGPD) |
| Uma TV, um app | Indoor hoje não tem WebSocket/SSE — fila exige latência baixa |
| Menos deploys no curto prazo | Quebra o posicionamento comercial já documentado |
| | Clínicas sem indoor precisariam comprar indoor para ter fila |
| | Evolução de um produto passa a travar o outro |

**Veredito:** só faz sentido como **widget opcional na mesma TV** (split screen: mídia + última chamada), não como fusão de produto.

### Opção B — Módulo dentro do web-agenda

Rota pública `/display/:code` na agenda + botão "Chamar" no calendário/recepção.

| Prós | Contras |
|---|---|
| Dados já estão na agenda (appointments, status 8, salas) | Agenda é app pesado, SSO, backoffice — superfície pública é outro contexto |
| Menos integração cross-app | Totem/autoatendimento poluiria o domínio de agendamento |
| | Deploy da agenda vira risco para TV da recepção |
| | Dificulta vender só "painel" sem módulo agenda completo |

**Veredito:** a **agenda é dona dos dados**, mas **não deve ser dona do produto de display**.

### Opção C — Novo app separado (recomendado)

Novo `web-queue` com API própria, módulo próprio no plano, host próprio.

| Prós | Contras |
|---|---|
| Bounded context claro (fila + display + totem) | Novo repo, deploy, catálogo, Statuspage, Loki |
| Comercialização independente (`queue` no plano) | Integração com agenda na v1 |
| Tempo real isolado (WS/SSE) sem afetar indoor/agenda | |
| LGPD e auditoria de "quem foi chamado na TV" no lugar certo | |
| Segue decomposição por domínio Octor | |

**Veredito:** **produto separado**, integrado com agenda (e opcionalmente com indoor na mesma TV).

---

## Recomendação de arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│  NOVO: web-queue (chamadas.octor.com.br)                        │
│  ├── Gestão (SSO): filas, guichês, salas, config LGPD, totem   │
│  ├── Operação (SSO): recepção chama / re-chama / pula           │
│  ├── Display (público): /display/:code — TV sem login           │
│  └── Totem (público, fase 2): /totem/:code — emite senha       │
└───────────────┬─────────────────────────────┬─────────────────────┘
                │ eventos / leitura          │ widget opcional
                ▼                            ▼
        ┌───────────────┐            ┌───────────────┐
        │  web-agenda   │            │  web-indoor   │
        │ status 8,     │            │  mídia na TV  │
        │ appointments, │            │  (PiP / split)│
        │ salas         │            └───────────────┘
        └───────────────┘
```

- **Não fundir com indoor.**
- **Não embutir na agenda.**
- Criar app novo com integração explícita.

---

## Escopo por fase

### Fase 1 — MVP "Painel de chamadas" (4–6 semanas)

**Personas:** secretária chama; paciente vê na TV.

| Camada | Entregável |
|---|---|
| **Recepção** | Lista do dia: pacientes com status `awaiting` (8) ou check-in manual na fila |
| **Ação** | "Chamar" → nome (ou iniciais) + sala/guichê/profissional na TV |
| **Display TV** | Rota pública `/display/:code`, fullscreen, atualização em tempo real |
| **Config** | Privacidade: nome completo / primeiro nome / iniciais; tempo na tela; som opcional |
| **Integração agenda** | Webhook ou poll: appointment → fila; ou entrada manual se sem agenda |

**Fora do MVP:** totem, impressora, TTS, múltiplas filas por especialidade.

### Fase 2 — Totem / senha

- Tablet na entrada: paciente identifica (CPF, QR do agendamento, ou nome)
- Emite senha ou entra na fila do profissional
- Impressora térmica (opcional)

### Fase 3 — Experiência completa

- Múltiplas filas (consultório, exame, caixa)
- Re-chamada, "não compareceu", prioridade
- Áudio / TTS ("Maria, consultório 3")
- **PiP com indoor:** faixa de chamada sobre a playlist (integração, não merge de apps)

---

## Modelo de dados (esboço)

Tabelas novas em schema do app (`octor_queue` ou prefixo `tbl_queue_*`):

| Entidade | Campos principais |
|---|---|
| `queue_settings` | `company_uuid`, `display_code`, privacidade, som, layout |
| `queue_counters` | guichê, sala, profissional (soft delete) |
| `queue_tickets` | senha, paciente (ref ou snapshot), status, `called_at`, `counter_id` |
| `queue_calls` | histórico de chamadas (auditoria LGPD) |

Status do ticket: `waiting` → `called` → `serving` → `done` | `no_show` | `cancelled`.

**Integração agenda:** `appointment_uuid` opcional em `queue_tickets`; ao marcar status 8 na agenda, evento cria/atualiza ticket (idempotente).

Todas as entidades deletáveis: `deleted_at` + `deleted_by` (soft delete obrigatório Octor).

---

## Arquitetura técnica (padrão Octor)

| Item | Decisão |
|---|---|
| Repo | `0ctor/web-queue` (dual FE + API Rust) |
| Host | `chamadas.octor.com.br` ou `fila.octor.com.br` (a confirmar) |
| Módulo AppMenu | `queue` + role `queue` (gestão) e `queue_operator` (só chamar) |
| Auth | SSO para gestão/operação; display/totem **sem login** (só código) |
| Tempo real | SSE ou WebSocket na API (`/v2/queue/display/:code/stream`) |
| DB | Percona, user de app, soft delete obrigatório |
| Deploy | `platform-deploy-hook`, health live/ready, Statuspage, Loki |
| Privacidade | Snapshot do nome no ticket (não re-query ao vivo na TV se paciente for deletado) |

**Reuso do indoor (sem acoplar produto):**

- Mesmo padrão de `player_code` de 6 caracteres
- Layout blank fullscreen
- Opcional: pacote compartilhado `@octor/tv-shell` (futuro)

---

## Comercialização

| Pacote | Módulos |
|---|---|
| **Básico** | `queue` — painel + recepção |
| **Recepção+** | `queue` + `indoor` — TV com mídia + chamadas (integração PiP) |
| **Sem agenda** | Fila manual/totem only (clínicas sem Octor Agenda) |

Não obrigar indoor para ter fila. Não obrigar fila para ter indoor.

---

## Decisões a fechar antes da implementação

1. **Fonte da fila na v1:** só appointments do dia, só check-in manual, ou ambos?
2. **O que aparece na TV:** nome completo é default aceitável ou iniciais por padrão (LGPD)?
3. **Escopo do totem:** entra na v1 ou fica fase 2?
4. **Nome do produto:** `web-queue` / Painel de Chamadas / Fila Octor / Recepção Octor?
5. **Host:** `chamadas.octor.com.br` vs `fila.octor.com.br`?

---

## Resumo executivo

| Pergunta | Resposta |
|---|---|
| Novo `web-{app}`? | **Sim** — `web-queue` |
| Dentro do indoor? | **Não** como produto único; no máximo widget na mesma TV |
| Produto separado? | **Sim** — módulo e deploy separados, integrado com agenda |
| Quem é dono dos dados? | **Agenda** (appointments, status 8, salas) |
| Quem é dono da experiência TV/totem? | **web-queue** |
