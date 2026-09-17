# web-queue

**Painel de chamadas / fila de atendimento** para clínicas Octor — TV da recepção, check-in na agenda e chamada pelo médico no prontuário.

| Item | Valor |
|---|---|
| Status | Fase 1 — sala de espera / TV |
| Host proposto (PRD) | `chamadas.octor.com.br` |
| DEV VPN (sem DNS) | TV `http://10.8.0.9:15177/display/:code` |
| API local | `http://localhost:4545` |
| TV local | `http://localhost:5178/display/:code` |
| Arquitetura | Dual FE + API Rust |

## Fluxo

1. Recepção na **web-agenda** (`/waiting-room`) marca **Paciente chegou** (status 8).
2. Médico no **web-medical-record** (dashboard) vê a fila e **chama** o próximo ou um paciente específico.
3. A TV (`/display/:code`) anuncia o nome ofuscado (LGPD).

## Como rodar

```bash
# schema (platform-database)
# mysql ... < schemas/octor_legacy/migrations/web-queue_20260917_tbl_queue.sql

cd backend/v2 && cargo test && cargo run
cd frontend && npm ci && npm test && npm run dev
```

Variáveis da API: `DB_USER`, `DB_PASS`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `BACKEND_PORT` (default 4545).

## Documentação

- **[PLANNING.md](./PLANNING.md)** — ADR e fases
- Issue canônica: [web-queue#21](https://github.com/0ctor/web-queue/issues/21)
