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
> Vale para **qualquer** assistente (Cursor, Claude, Codex, ChatGPT, etc.) e para humanos.
> Detalhe canônico: portal [`0ctor/backstage`](https://github.com/0ctor/backstage).

1. **Git Flow:** proibido editar/push em `main`. `feature/*` ← `dev` → PR→`dev` → PR `dev`→`main` (deploy **só** em `main`). Hotfix: `fix/*` ← `main` → PR→`main` + devolver a `dev`. Apagar branch após merge (remota + local). Doc: [git-flow.md](https://github.com/0ctor/backstage/blob/main/git-flow.md).
2. **Schema (ADR-005):** DDL/migrations **somente** em [`platform-database`](https://github.com/0ctor/platform-database). Proibido `Schema::` / `dbforge` / pastas `migrations/` de schema em outros apps.
3. **Auth / SSO:** sempre [`web-auth`](https://github.com/0ctor/web-auth) (`auth.octor.com.br`). Não reinventar portal de login.
4. **Segredos:** só `.env` / Vault — nunca commit. GitHub Actions: **Variables** para não-sensível (URLs, portas); **Secrets** para credenciais.
5. **Loki:** backend `LOKI_PUSH_*` + `LOKI_JOB` no `.env` da API; frontend via `POST /v1/errors/report` (ou `/v2/...`) na API — **proibido** senha Loki em `NEXT_PUBLIC_*`.
6. **Constraints:** sem hostname/IP de produção hardcoded no código de app; **sem** rotas HTTP `OPTIONS` (CORS no gateway); soft delete `deleted_at` + `deleted_by`.
7. **Deploy:** push/`main` → `platform-deploy-hook` → GHCR `ghcr.io/0ctor/<app>:<sha>` → compose em [`sp1-sd-octor-1`](https://github.com/0ctor/sp1-sd-octor-1) (`apps/<serviço>/`).
8. **Testes (mínimo):** unitários obrigatórios; CI `lint → typecheck → test → build` em **`dev` e `main`**. APIs: integração; apps de usuário: E2E/smoke (SSO + fluxo principal + anti–tela branca). Metas: coverage ≥ 80%; mutation ≥ 80% quando configurado. Canônico: [testing-strategy.md](https://github.com/0ctor/backstage/blob/main/testing-strategy.md).
9. **Agente:** username **Alfred** (nunca “Mordomo Octor”).
10. **Stack:** respeitar a do repo — não trocar framework sem decisão explícita.
11. **Mapa cross-app:** [`catalog.yaml`](https://github.com/0ctor/backstage/blob/main/catalog.yaml) / [catalog-rede](https://github.com/0ctor/backstage/blob/main/catalog-rede.json). UX → sync [`platform-help`](https://github.com/0ctor/platform-help). App novo: [novo-app-octor.md](https://github.com/0ctor/backstage/blob/main/novo-app-octor.md).
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