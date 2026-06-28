# CHANGELOG — Koda

> Format : [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/)
> Versioning : [SemVer](https://semver.org/lang/fr/)

---

## [Unreleased]

### Added
- OpenAPI 3.1 / Swagger UI : `utoipa` v4 + `utoipa-swagger-ui` v7 montés sur `/swagger-ui` et `/api-docs/openapi.json` — endpoints documentés, sécurité cookie session, 15 tags
- `rate_limit_middleware` branché en layer global : 300 req/min par IP, 600 req/min par utilisateur authentifié, Redis sliding window INCR + EXPIRE
- Tests E2E Playwright : 5 correctifs — répertoire `.auth/` manquant (ENOENT), regex label français `/^nom$/i`, mode `serial` pour `workspaceId` partagé, `e2e/.gitignore`, middleware rate limiting absent des routes
- Filtre secrets : `is_secret_file()` côté API et client — `.env`, `*.key`, `*.pem`, clés SSH jamais transmises au LLM
- Packs framework built-in : `axum`, `react`, `nextjs`, `sqlx` (Markdown, non supprimables) + `builtin_framework_pack()`
- Auto-détection packs depuis extension du fichier courant (`.rs` → rust+axum+sqlx, `.tsx` → typescript+react+nextjs)
- API git stubs : `GET .../git/status`, `POST .../git/stage`, `POST .../git/commit`, `POST .../git/push`
- API MCP workspace : `GET .../mcp/connectors`, `GET|POST .../mcp/bindings`, `DELETE .../mcp/bindings/:id`
- `workspace_notes` : migration DB + `GET|PUT /api/v1/organizations/:org_id/workspaces/:workspace_id/notes` (par utilisateur, upsert)
- Routes TCP sozu : `add_workspace_tcp_route` / `remove_workspace_tcp_route` dans orchestrateur et gateway (SSH 2200–2999, Postgres 5400–5499)
- IDE responsive : détection mobile/tablette/desktop, mode `full-ide | tablet-ide | mobile-view`
- `GitPanel` : composant web-client avec status git, stage/unstage, commit, push
- `McpPanel` : composant web-client avec liste connecteurs, activation/désactivation, suppression
- Dashboard workspace : onglet « Diff » (stub Phase 3)
- i18n exhaustive : messages `ide`, `mcp`, `git`, `personal` dans les 4 langues (FR/EN/ES/DE) — dashboard + web-client (`packages/i18n/messages/`)
- **Phase 3 — correctifs et complétions** :
  - Correction critique : `post_pipeline_run` publie maintenant dans le stream Redis `koda:jobs:pipeline` — le worker exécute effectivement les pipelines
  - Correction critique : `enqueue_push_pipelines` publie dans Redis pour les triggers `on_push`
  - Correction bug : `workspaces.deleted_at` inexistant remplacé par `status != 'closed'` dans tous les handlers (`post_webhook`, `get_webhook_events`, `admin.rs`, `garbage_collector.rs`)
  - API historique d'exécution : `GET .../pipelines/{pipeline_id}/runs` — liste des jobs avec status/error/attempts
  - API activité workspace : `GET .../workspaces/{workspace_id}/activity` — feed `audit_events` filtré par workspace
  - Dashboard : onglet Activité avec feed réel, lien « Historique » par pipeline
  - `run_sast` : implémentation réelle OWASP Top 10 via LLM Anthropic (claude-haiku) — findings parsés et sauvegardés en DB
  - `run_container_pipeline` : implémentation bollard — container éphémère avec resource limits, labels koda, wait, collecte logs, cleanup
  - `SecurityPolicy.min_severity_to_block` : enforcement post-pipeline — workspace passe en `reviewing` si seuil atteint
  - Branches éphémères pipeline `pipeline/<uid>/<timestamp>` via git2 (fallback gracieux si pas de repo cloné)
  - `WorkerConfig.anthropic_api_key` : nouveau champ optionnel pour le SAST LLM
  - `PipelineRunner` : ajout `http`, `docker_host`, `anthropic_api_key` au struct
  - `JobRun` type dans `@koda/shared-types`

---

## [0.4.0] — 2026-06-27 · Phase 3 — Pipelines CI/CD

### Added
- Modèles DB : `CiCdPipeline`, `AutomationTrigger`, `IncomingWebhookEvent`, `Job`
- `pipeline_runner.rs` : exécution réelle de tous les types de pipeline
- `secret_scan` : parcours de fichiers par walkdir + regex patterns + entropie Shannon pour détecter les tokens hardcodés
- `dependency_scan` : `cargo audit --json` + `npm audit --json` avec parsing des CVE et mapping CVSS → severity
- `sast` : rapport LLM dédié OWASP Top 10, severity scoring Critical/High/Medium/Low/Info
- `image_scan` : exécution Trivy/Grype via container éphémère
- `build` / `lint` : exécution dans containers éphémères avec resource limits
- `cron_scheduler.rs` : évaluateur cron 5 champs (min, hour, dom, month, dow) + step `*/n` + ranges
- Triggers `on_push`, `schedule`, `manual` via `AutomationTrigger`
- Webhook entrant : vérification HMAC-SHA256 + stockage `IncomingWebhookEvent`
- Dead-letter stream : jobs échoués après 3 tentatives → `koda:jobs:pipeline:dead`
- API : endpoints pipelines, triggers, pipeline run, webhook events, security reports
- `SecurityReport`, `VulnerabilityFinding`, `SecurityPolicy`, `ScanRule` (DB + API)
- Rapport sécurité consultable par workspace (`GET .../security-reports`)
- `KodaInstance` + `OrgInstanceAffinity` : fondations multi-instances (migration DB)

---

## [0.3.0] — 2026-06-27 · Phase 2 — Workspace complet

### Added
- `apps/web-client/` : IDE web avec Monaco Editor + chat IA sidebar
- `ide::get_workspace_files` + `get_workspace_file_content` : navigation fichiers workspace
- `ide::post_workspace_ai_chat` : endpoint chat IA SSE avec 5 niveaux de prompt (nano, quick, standard, deep, agent)
- `ai/context_builder.rs` : assemblage 6 couches de contexte LLM (platform → org → lang packs → framework packs → KODA.md → ai/instructions.md)
- Packs langue built-in : `rust`, `typescript`, `python`, `go`, `sql` (Markdown, non supprimables)
- `plugin_prober.rs` : health probe HTTP par binding actif, mise à jour `health_status`
- `PluginDefinition` + `WorkspacePluginBinding` : migrations + modèles DB
- Catalogue plugins : `koda-web-ide`, `code-server`, `ssh`, `jupyter`
- `UserMCPBinding` : connecteurs MCP personnels distincts des bindings workspace
- `PersonalSpace` : volume Docker monté en `:ro` dans chaque workspace (`personal_spaces`, `personal_snippets`)
- API snippets personnels : CRUD complet (`GET|POST|PATCH|DELETE /api/v1/personal/snippets`)
- `services/mcp-gateway/` : service Rust Redis Streams consumer, trait `McpConnector`, registry auto-enregistrement
- Connecteurs built-in mcp-gateway : jira, notion, postgres, slack, http (Rust)
- `packages/mcp-connectors/` : registre TypeScript + connecteurs built-in (jira, notion, postgres, slack, http)
- `MCPConnectorDefinition` + `WorkspaceMCPBinding` : migrations DB
- `SecretRef` : résolution credentials au moment du `call_tool`, jamais loggué
- `KODA.md` support : fichier workspace-level LLM-agnostique (couche 5 du contexte)
- `UserSettings.locale` injecté en couche 6 du contexte LLM via `AiContextBuilder`
- `user_settings` : API `GET|PUT /api/v1/user/settings` (locale, theme_id)
- i18n : `packages/i18n/` + `next-intl` sur les 3 apps Next.js + messages FR/EN/ES/DE

---

## [0.2.0] — 2026-06-27 · Phase 1 — Workspace minimal

### Added
- `services/orchestrator/` : orchestration Docker complète via bollard + docker-socket-proxy
- `docker.rs` : `start_workspace`, `stop_workspace`, `delete_workspace`, `ensure_personal_volume`
- Resource limits systématiques : `cpu_period`, `cpu_quota`, `memory`, `pids_limit` (invariant)
- Labels `koda.*` obligatoires : `koda.managed`, `koda.type`, `koda.workspace_id`, `koda.org_id`
- Volume personnel monté en `:ro` à chaque démarrage de workspace
- `cap_drop: ALL` + `no-new-privileges:true` sur tous les containers
- Modèles DB : `Workspace`, `WorkspaceGitConfig`, `WorkspaceVolume`, `Template`, `Project`
- API workspace : `POST|GET|DELETE /api/v1/organizations/:org_id/workspaces`
- API start/stop : `POST .../workspaces/:id/start`, `POST .../workspaces/:id/stop`
- `WorkspaceShare` : partage ad-hoc (editor|reviewer|viewer) avec expiration
- `services/worker/` : `garbage_collector.rs` (GC volumes orphelins toutes les heures) + `prewarm_images` (daily)
- Snapshots workspace : `POST|GET .../workspaces/:id/snapshots`
- Réseaux Docker : `koda-ws-<uid>-internal` + `koda-ws-<uid>-services` (stratégie)

---

## [0.1.0] — 2026-06-27 · Phase 0 — Fondations

### Added
- Monorepo : `apps/`, `services/`, `packages/`, `infra/`, `docs/`
- Workspace Cargo multi-crates : api, orchestrator, worker, git-manager, gateway, mcp-gateway
- API Axum : inscription, connexion, OAuth Google/GitHub/Authentik OIDC, sessions cookie HttpOnly
- `require_auth`, `require_super_admin`, `with_org_context` middlewares
- PostgreSQL + sqlx-migrate : `Organization`, `User`, `Membership`
- RBAC Teams : `Team`, `TeamMembership`, `TeamProjectAccess`, `TeamQuota`
- `WorkspaceShare` : modèle DB + API
- Rôle `super_admin` + bootstrap `BOOTSTRAP_SUPER_ADMIN_EMAIL`
- `apps/admin/` : panel super_admin (organisations, utilisateurs, impersonation, audit logs, stats infra)
- `AuditEvent` : traçabilité actions critiques — impersonation tracée
- TOTP MFA : setup, verify, status, delete (`/api/v1/user/mfa/*`)
- Tokens M2M : `POST|GET|DELETE /api/v1/organizations/:org_id/tokens`
- `SecretRef` : modèle DB + AES-256-GCM
- Trait `AiProviderAdapter` + implémentation Anthropic HTTP (reqwest)
- `services/gateway/` : client sozu-command-lib (add/remove HttpFrontend + TcpFrontend)
- `infra/sozu/sozu.toml` : configuration sozu Docker Compose dev
- `apps/dashboard/` : Next.js — login, register, liste workspaces, création workspace, settings, page workspace
- `packages/themes/` : `ThemeRegistry`, `ThemeSwitcher`, 4 skins (default, minimal, pro, light)
- `packages/shared-types/` + `packages/api-client/` : client HTTP typé
- Config figment par service : `config/default.yaml` + `.env.example` (merge YAML → env → .env)
- `TRUSTED_PROXY_CIDRS` + `axum-client-ip` sur tous les services Axum
- RLS PostgreSQL activé sur 13 tables critiques (`202600010035_enable_rls.sql`)
- 38 migrations DB couvrant toutes les entités

### Security
- `rate_limit_middleware` : 300 req/min par IP, 600 req/min par utilisateur (Redis)
- Sessions cookie : HttpOnly, SameSite=Strict, expiration configurable
- Argon2id pour les mots de passe
- HMAC-SHA256 pour les webhooks entrants
