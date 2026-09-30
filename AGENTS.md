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

1. **Git Flow:** proibido editar/push em `main`. `feature/*` ← `dev` → PR→`dev` → PR `dev`→`main` (deploy **só** em `main`). **Hotfix absoluto** (bug que precisa ir já a `main`): `fix/*` ← `main` → PR→`main` + PR/cherry-pick do mesmo fix em `dev` — proibido acelerar via `dev`+promote. Apagar branch após merge (local; remota via setting GitHub **`delete_branch_on_merge`** em todos os repos — script [ensure_delete_branch_on_merge.py](https://github.com/0ctor/backstage/blob/main/scripts/ensure_delete_branch_on_merge.py)). **Proibido** usar `git stash` como depósito de WIP/funcionalidade — código escrito vive em `feature/*`/`fix/*` com commit (troca de tarefa: commit ou worktree). Doc: [git-flow.md](https://github.com/0ctor/backstage/blob/main/git-flow.md).
2. **Schema (ADR-005):** DDL/migrations **somente** em [`platform-database`](https://github.com/0ctor/platform-database). Proibido `Schema::` / `dbforge` / pastas `migrations/` de schema em outros apps.
3. **Auth / SSO:** sempre [`web-auth`](https://github.com/0ctor/web-auth) (`auth.octor.com.br`). Não reinventar portal de login.
4. **Segredos:** Vault OPS = fonte de verdade; `.env` no disco é **só materialização** (`~/.octor/run/<app>/env` + symlink `./.env`) — nunca commit. Config não-secreta: compose / Variables. Actions: **Variables** (URLs, portas) vs **Secrets** (credenciais). Ao tocar deploy/`.env`/compose: pagar o padrão **nessa app** (DEV → depois PRD no promote). Canônico: [secrets-vault.md](https://github.com/0ctor/backstage/blob/main/secrets-vault.md) · CLI [`octor-env`](https://github.com/0ctor/backstage/blob/main/scripts/octor-env).
5. **Loki:** backend `LOKI_PUSH_*` + `LOKI_JOB` no `.env` da API; frontend via `POST /v1/errors/report` (ou `/v2/...`) na API — **proibido** senha Loki em `NEXT_PUBLIC_*`.
6. **Constraints:** sem hostname/IP de produção hardcoded no código de app; sem gates absolutos de PRD no código (usar env / feature flag / config de deploy — não `if (host===…)`); **sem** rotas HTTP `OPTIONS` (CORS no gateway); soft delete `deleted_at` + `deleted_by`. Proibido logar ou colocar em URL token, CPF, telefone, nome ou conteúdo clínico.
7. **Deploy:** push/`main` → `platform-deploy-hook` → GHCR `ghcr.io/0ctor/<app>:<sha>` → compose em [`sp1-sd-octor-1`](https://github.com/0ctor/sp1-sd-octor-1) (`apps/<serviço>/`).
8. **Testes (mínimo):** unitários obrigatórios; CI `lint → typecheck → test → build` em **`dev` e `main`**. APIs: integração; apps de usuário: E2E/smoke (SSO + fluxo principal + anti–tela branca). Metas: coverage ≥ 80%; mutation ≥ 80% quando configurado. Canônico: [testing-strategy.md](https://github.com/0ctor/backstage/blob/main/testing-strategy.md).
9. **Agente:** username **Alfred** (nunca “Mordomo Octor”).
10. **Stack:** respeitar a do repo — não trocar framework sem decisão explícita.
11. **Mapa cross-app:** [`catalog.yaml`](https://github.com/0ctor/backstage/blob/main/catalog.yaml) / [catalog-rede](https://github.com/0ctor/backstage/blob/main/catalog-rede.json). Mudança de contrato / portas / API / arquitetura → atualizar o Backstage ([docs-sync.md](https://github.com/0ctor/backstage/blob/main/docs-sync.md)). UX → sync [`platform-help`](https://github.com/0ctor/platform-help). App novo: [novo-app-octor.md](https://github.com/0ctor/backstage/blob/main/novo-app-octor.md).
12. **Dependentes de dados:** ao mudar schema/contrato/semântica de tabelas, avaliar e atualizar [`web-migration`](https://github.com/0ctor/web-migration), [`web-export`](https://github.com/0ctor/web-export) e demais consumidores do mesmo dado (ex. `web-database`, `platform-legacy`, apps do domínio) — ou registrar explicitamente “sem impacto”. Detalhe: [cross-app-data-dependents-sync](https://github.com/0ctor/backstage/blob/main/.cursor/rules/cross-app-data-dependents-sync.mdc).
13. **Datas / fusos:** instantes de auditoria (`created_at`/`updated_at`/`deleted_at`) em **UTC** no backend; API com ISO `Z`/offset (naive legado = UTC). UI no fuso do operador; se mostrar UTC, sufixo ` UTC`. Proibido `timeZone="UTC"` no next-intl sem rótulo e `Local::now()`/`date()` do servidor para auditoria. Agenda/slots/vencimentos = civil (helpers separados). Canônico: [timezone-datetime.md](https://github.com/0ctor/backstage/blob/main/timezone-datetime.md).
14. **Skills / agentes (org):** política = este bloco. Procedimentos = [`agent-skills/`](https://github.com/0ctor/backstage/tree/main/agent-skills). No dia a dia: **`make up` / `make agents-sync`** atualiza `AGENTS.md` + skills no laptop (script [`octor-agents-sync`](https://github.com/0ctor/backstage/blob/main/scripts/octor-agents-sync)). Preferência pessoal de IDE **não** entra no canônico. Doc: [agent-skills.md](https://github.com/0ctor/backstage/blob/main/agent-skills.md).
15. **Antes de começar:** regra absoluta para issue nova, branch nova ou comportamento novo. Buscar neste repo (issue aberta e fechada, branch, PR, código, parcial). Catálogo ou app irmão se a capacidade pode viver noutro repo. Issue aberta: continuar, sem duplicata. Issue fechada: não reabrir no escuro. Branch: continuar só se for o mesmo trabalho e a base estiver certa (`dev`, ou `main` no hotfix); branch alheia viva não se toma; `git stash` não conta. PR mergeada: não recriar a branch. Plano de poucas linhas com o que achou; parar só se houver duas opções reais. Canônico: [antes-de-comecar.md](https://github.com/0ctor/backstage/blob/main/antes-de-comecar.md).
16. **Erros de API na UI:** ao tocar fluxo HTTP de app de operador, a tela mostra a mensagem de negócio que a API já enviou, pelo extrator que o repositório já tem (sem nome obrigatório de helper). Sem resposta HTTP, a UI diz que a conexão falhou e, na escrita, que nada foi concluído. Fallback genérico só quando não houver mensagem utilizável. Proibido substituir o motivo por texto fixo, exibir `Request failed with status code`, `catch` vazio e repassar detalhe interno de 500. Canônico: [api-error-messages.md](https://github.com/0ctor/backstage/blob/main/api-error-messages.md).
17. **Testes antes do deploy (absoluto):** em qualquer repo da org, se a suíte existe e o piso desse repo é **100%** (lines, statements, branches, functions e pass rate; mutação quando o repo exige), é proibido deploy sem ter **rodado** essa suíte nesta alteração e ela ter passado. Vale para hot-deploy, imagem que publica, compose da imagem nova, merge do promote `dev`→`main` e para afirmar que subiu. CI verde de outro commit não conta se o tree mudou. Sem essa suíte: não fingir que os testes rodaram. Canônico: [testing-strategy.md](https://github.com/0ctor/backstage/blob/main/testing-strategy.md).
18. **Tickets:** comentário público (`is_internal=0`) no web-ticket aparece para o cliente. Não comentar salvo pedido explícito. Se for pedido: português de atendimento, sem hotfix, PR, SHA, SQL, rota interna, status de importação ou checklist de migração. Nota de time: `is_internal=1`. Status operacional fica no chat interno ou na issue — não no fio público.
19. **Diagramas:** arquitetura, workflow, sequência, fluxo de dados e ciclo de vida usam [Archify](https://github.com/tt-a1i/archify) (JSON + HTML autônomo). Backstage: `diagramas/archify/`. Outros repos: `docs/diagramas/`. Texto autoral em português; UI do viewer em inglês. Validar antes de entregar. O mapa do catálogo continua em `diagramas/_platforms.py`. Canônico: [docs-sync.md](https://github.com/0ctor/backstage/blob/main/docs-sync.md).
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