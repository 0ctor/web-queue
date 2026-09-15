# AGENTS.md — `web-queue`

Este repositório faz parte da plataforma **Octor** (org [`0ctor`](https://github.com/0ctor)).
Leia isto **antes** de mudanças de arquitetura, auth, deploy ou padrões compartilhados.

## Este componente

| Campo | Valor |
|-------|-------|
| Domínio | `ia` |
| Sistema | `unknown` |
| Papel | `Service` |

- **Host público:** (ver Traefik / docs do repo)
- **Deploy:** serviço/imagem única (ou conforme compose do repo)
- **Notas:**

## Padrões da org (obrigatórios)

<!-- octor-ecosystem:start -->
> Vale para **qualquer** assistente e IDE (**Cursor**, **OpenCode**, **Codex**, **VS Code**/Copilot, Claude, ChatGPT, etc.) e para humanos.
> **Fonte portátil (igual para todo o time):** este `AGENTS.md` — o bloco abaixo. Pastas de IDE (`.cursor/`, `.opencode/`, `.codex/`, …) são **opcionais** e **não** podem contradizer nem substituir este bloco.
> Detalhe canônico: portal [`0ctor/backstage`](https://github.com/0ctor/backstage) (`agents-ecosystem-block.md` + docs).

1. **Git Flow:** proibido editar/push em `main`. `feature/*` ← `dev` → PR→`dev` → PR `dev`→`main` (deploy **só** em `main`). **Hotfix absoluto** (bug que precisa ir já a `main`): `fix/*` ← `main` → PR→`main` + PR/cherry-pick do mesmo fix em `dev` — proibido acelerar via `dev`+promote. Apagar branch após merge (remota + local). Doc: [git-flow.md](https://github.com/0ctor/backstage/blob/main/git-flow.md).
2. **Schema (ADR-005):** DDL/migrations **somente** em [`platform-database`](https://github.com/0ctor/platform-database). Proibido `Schema::` / `dbforge` / pastas `migrations/` de schema em outros apps.
3. **Auth / SSO:** sempre [`web-auth`](https://github.com/0ctor/web-auth) (`auth.octor.com.br`). Não reinventar portal de login.
4. **Segredos:** só `.env` / Vault — nunca commit. GitHub Actions: **Variables** para não-sensível (URLs, portas); **Secrets** para credenciais.
5. **Loki:** backend `LOKI_PUSH_*` + `LOKI_JOB` no `.env` da API; frontend via `POST /v1/errors/report` (ou `/v2/...`) na API — **proibido** senha Loki em `NEXT_PUBLIC_*`.
6. **Constraints:** sem hostname/IP de produção hardcoded no código de app; sem gates absolutos de PRD no código (usar env / feature flag / config de deploy — não `if (host===…)`); **sem** rotas HTTP `OPTIONS` (CORS no gateway); soft delete `deleted_at` + `deleted_by`.
7. **Deploy:** push/`main` → `platform-deploy-hook` → GHCR `ghcr.io/0ctor/<app>:<sha>` → compose em [`sp1-sd-octor-1`](https://github.com/0ctor/sp1-sd-octor-1) (`apps/<serviço>/`).
8. **Testes (mínimo):** unitários obrigatórios; CI `lint → typecheck → test → build` em **`dev` e `main`**. APIs: integração; apps de usuário: E2E/smoke (SSO + fluxo principal + anti–tela branca). Metas: coverage ≥ 80%; mutation ≥ 80% quando configurado. Canônico: [testing-strategy.md](https://github.com/0ctor/backstage/blob/main/testing-strategy.md).
9. **Agente:** username **Alfred** (nunca “Mordomo Octor”).
10. **Stack:** respeitar a do repo — não trocar framework sem decisão explícita.
11. **Mapa cross-app:** [`catalog.yaml`](https://github.com/0ctor/backstage/blob/main/catalog.yaml) / [catalog-rede](https://github.com/0ctor/backstage/blob/main/catalog-rede.json). Mudança de contrato / portas / API / arquitetura → atualizar o Backstage ([docs-sync.md](https://github.com/0ctor/backstage/blob/main/docs-sync.md)). UX → sync [`platform-help`](https://github.com/0ctor/platform-help). App novo: [novo-app-octor.md](https://github.com/0ctor/backstage/blob/main/novo-app-octor.md).
12. **Dependentes de dados:** ao mudar schema/contrato/semântica de tabelas, avaliar e atualizar [`web-migration`](https://github.com/0ctor/web-migration), [`web-export`](https://github.com/0ctor/web-export) e demais consumidores do mesmo dado (ex. `web-database`, `platform-legacy`, apps do domínio) — ou registrar explicitamente “sem impacto”. Detalhe: [cross-app-data-dependents-sync](https://github.com/0ctor/backstage/blob/main/.cursor/rules/cross-app-data-dependents-sync.mdc).
<!-- octor-ecosystem:end -->


## Mapa da plataforma (sob demanda)

- Catálogo: [`0ctor/backstage`](https://github.com/0ctor/backstage) (`catalog.yaml`)
- Ops host: [`sp1-sd-octor-1` AGENTS.md](https://github.com/0ctor/sp1-sd-octor-1/blob/main/AGENTS.md)
- Loki (ref.): `web-auth` → `docs/LOKI_OBSERVABILITY.md`

## Como o agente deve trabalhar **neste** repo

1. Mudanças pequenas, alinhadas ao código existente.
2. PR para `dev` (branch atual de feature → `dev`).
3. Compose/env no servidor → `sp1-sd-octor-1` (`apps/web-queue/`) quando for o caso.
4. Dependência cross-app → consultar catálogo / AGENTS do host.