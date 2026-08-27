# web-queue

**Painel de chamadas / fila de atendimento** para clínicas Octor — TV da recepção, operação na secretaria e totem (fases futuras).

| Item | Valor |
|---|---|
| Status | Planejamento (sem implementação) |
| Host proposto | `chamadas.octor.com.br` ou `fila.octor.com.br` |
| Módulo AppMenu | `queue` |
| Arquitetura | Dual FE + API Rust (padrão Octor) |

## Documentação

- **[PLANNING.md](./PLANNING.md)** — planejamento completo (ADR, fases, modelo de dados, decisões pendentes)
- Issues no GitHub — backlog detalhado por épico e entregável

## Relação com outros produtos

| Produto | Papel |
|---|---|
| **web-agenda** | Fonte de dados (appointments, status 8 "chegou", salas) |
| **web-indoor** | Signage na mesma TV (integração PiP opcional — **não** fusão de produto) |
| **web-auth** | SSO + entitlement do módulo `queue` |

## Decisão de produto

App **separado** (`web-queue`), não extensão do indoor nem módulo embutido na agenda.
Ver análise completa em [PLANNING.md](./PLANNING.md).
