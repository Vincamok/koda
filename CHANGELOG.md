# CHANGELOG — Koda

> Format : [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/)
> Versioning : [SemVer](https://semver.org/lang/fr/)

---

## [Unreleased]

### Added
- OpenAPI 3.1 / Swagger UI : `utoipa` v4 + `utoipa-swagger-ui` v7 montés sur `/swagger-ui` et `/api-docs/openapi.json` — 58 endpoints documentés, sécurité cookie session, 13 tags, `ApiDoc` centralisé dans `openapi.rs`
- `rate_limit_middleware` branché en layer global sur toutes les routes (300 req/min par IP, 600 req/min par utilisateur authentifié, Redis sliding window INCR + EXPIRE)
- Tests E2E Playwright : 5 bugs corrigés — répertoire `.auth/` manquant (ENOENT), regex label français `/nom du workspace/` → `/^nom$/i`, mode `serial` pour état partagé `workspaceId`, `e2e/.gitignore` protège les tokens de session, middleware rate limiting absent des routes
- Interface administration (`apps/admin/`) : panel super_admin dédié (quotas, logs, IA, infra, multi-instances)
- Rôle `super_admin` (plateforme, non org-scoped) avec impersonation tracée
- `KodaInstance` + `OrgInstanceAffinity` : fondations multi-instances Koda depuis un panel central
- OAuth Authentik (OIDC générique) en plus de Google et GitHub
- `SecretRef` stockage : colonne DB chiffrée AES-256-GCM + inject Docker env pour secrets runtime
- `packages/shared-types/` + `packages/api-client/` (client HTTP typé généré depuis OpenAPI)
- `OrganizationQuota` : champs `max_workspaces`, `max_cpu_cores`, `max_ram_gb`, `max_storage_gb`, `max_members`
- `devcontainer.json` : lecture au clonage → pré-remplissage Template/Plugin (roadmap v0.3.0)
- Catalogue plugins validé : `koda-web-ide`, `code-server`, `ssh`, `jupyter`
- Plages TCP sozu validées : SSH `2200-2999`, PostgreSQL `5400-5499`
- Workspace `reviewing` → clôture libre (pas de blocage obligatoire)
- Backlog enrichi : KODA-B14 multi-instances avancé, KODA-B15 TicketRecord, KODA-B16 marketplace plugins
- Fix mcp-gateway : `base64_encode` partagé (bug compilation `http.rs`), dead-letter, XGROUP CREATE, figment, UserMCPBinding TypeScript
- Internationalisation (i18n) : `next-intl` sur les 3 apps Next.js + `packages/i18n/` clés partagées + 4 langues MVP (FR, EN, ES, DE)
- `UserSettings` : entité `user_settings` (locale, theme_id) + API `GET|PUT /api/v1/users/me/settings`
- Injection langue `UserSettings.locale` en couche 6 du contexte LLM via `AiContextBuilder`
- Pré-prompts LLM-agnostiques : hiérarchie 6 couches (platform → org → lang packs → framework packs → `KODA.md` → `ai/instructions.md`)
- Packs langue et framework built-in avec auto-détection depuis manifestes repo (Cargo.toml, package.json, etc.)
- `KODA.md` : fichier workspace-level LLM-agnostique (distinct de `CLAUDE.md` pour Claude Code)
- `ai/instructions.md` : renommage depuis `ai/CLAUDE.md` dans PersonalSpace
- Analyse de faisabilité initiale (`docs/FEASIBILITY_ANALYSIS.md`)
- Décisions architecturales : Rust (Axum + SQLx), sozu gateway, Harness CI/CD
- Système de thèmes évolutif : `ThemeRegistry` observable, `SkinManifest` avec héritage `extends`, `loadFromUrl()` marketplace
- 4 skins built-in (default, minimal, pro, light) avec layouts distincts
- `ThemeSwitcher` avec miniatures `SkinPreview` générées depuis les tokens CSS
- MCP connecteurs plugins : `@koda/mcp-connectors` (registre TypeScript + 6 connecteurs built-in)
- `MCPConnectorRegistry` observable (même pattern que ThemeRegistry)
- `services/mcp-gateway/` : service Rust consommant Redis Streams `jobs:mcp`, 6 connecteurs built-in
- `McpConnector` trait Rust + `ConnectorRegistry` auto-enregistrement
- RBAC Teams : `Team`, `TeamMembership`, `TeamProjectAccess`, `TeamQuota` (3 couches : org + team + workspace)
- `WorkspaceShare` : partage ad-hoc avec expiration (editor|reviewer|viewer)
- Stratégie réseaux Docker multi par workspace (`koda-ws-<uid>-internal`, `koda-ws-<uid>-services`, `koda-egress`)
- Labels `koda.*` obligatoires sur tous les containers éphémères (GC + traçabilité)
- `PersonalSpace` : espace personnel portable par utilisateur (volume Docker + 7 catégories de fichiers)
- `UserMCPBinding` : connecteurs MCP personnels (distinct des bindings workspace)
- 5 niveaux de prompt IA (nano, quick, standard, deep, agent) avec filtre secrets et audit
- Détection device → 3 modes IDE (full-ide, tablet-ide, mobile-view)
- Sécurité intégrée dans les projets : `SecurityReport`, `VulnerabilityFinding`, `SecurityPolicy`
- 4 types de scan CI/CD : secret_scan, sast (LLM dédié), dependency_scan, image_scan
- Config par service : `config/default.yaml` + `.env.example` + `figment` (merge YAML/env/.env)
- Proxy trust : `TRUSTED_PROXY_CIDRS` + `axum-client-ip` sur tous les services Axum
- Spécification d'implémentation par module (`docs/IMPLEMENTATION_SPEC.md`)
- Roadmap versionnée v0.1.0 → v1.0.0

---

## [0.1.0] — À venir · Phase 0

### Added
- Monorepo Rust multi-crates (api, orchestrator, worker, git-manager, gateway)
- API Axum : authentification (session cookie, OAuth Google/GitHub)
- PostgreSQL + sqlx-migrate : Organization, User, Membership, Team, TeamMembership
- PersonalSpace : modèle DB + volume Docker `koda-personal-<user-uid>` (fondations)
- Trait `AiProviderAdapter` + implémentation Anthropic
- sozu en Docker Compose dev
- Service gateway/ : client sozu-command-lib
- Dashboard Next.js : skeleton + login (responsive mobile-first)
- ThemeProvider + sélecteur de thèmes (4 skins)
- Config par service : `config/default.yaml` + `.env.example` + figment
- `TRUSTED_PROXY_CIDRS` + `axum-client-ip` sur API
- Pipeline Harness : CI toutes branches + déploiement prod auto sur main

---

## [0.2.0] — À venir · Phase 1

### Added
- Workspace : création, statut, UID immuable
- WorkspaceGitConfig : clone asynchrone (pending → cloning → ready | failed)
- WorkspaceVolume : cycle de vie (création, montage, détachement)
- WorkspaceShare : partage ad-hoc (editor|reviewer|viewer), expiration
- Container Docker via bollard + docker-socket-proxy (resource limits obligatoires)
- Réseaux Docker multi : `koda-ws-<uid>-internal` + `koda-ws-<uid>-services` + `koda-egress`
- Labels `koda.*` obligatoires sur tous les containers
- ExposureRule HTTP via sozu
- SSE : `/api/v1/workspaces/:uid/events`
- Dashboard : liste workspaces + création + statut temps réel
- Dashboard multi-device : responsive mobile (monitoring + start/stop)

---

## [0.3.0] — À venir · Phase 2

### Added
- PluginDefinition catalogue (koda-web-ide, code-server, ssh, jupyter)
- WorkspacePluginBinding + health probe par plugin
- Koda Web IDE : Monaco Editor + file tree + terminal xterm.js + chat IA sidebar + git panel
- **5 niveaux de prompt IA** (nano, quick, standard, deep, agent) + filtre secrets + détection prompt injection
- **Détection device** → 3 modes IDE (full-ide | tablet-ide | mobile-view)
- **PersonalSpace complet** : volume monté en read-only, fusion CLAUDE.md, snippets, shell/git configs, notes workspace
- Panel "Mon espace" dans web-client (édition Monaco de tous les fichiers `.personal/`)
- UserMCPBinding : connecteurs MCP personnels
- Endpoints API fichiers : `GET|PUT /api/v1/workspaces/:uid/files/*`
- Endpoint chat IA : `POST /api/v1/workspaces/:uid/ai/chat` (SSE streaming, body inclut `prompt_level`)
- MCP connecteurs dans le web-client : panel activation, config, statut
- MCPConnectorDefinition + WorkspaceMCPBinding + UserMCPBinding (DB + API)
- mcp-gateway Rust opérationnel (6 connecteurs : github, jira, notion, postgres, slack, http)
- Injection tool definitions MCP workspace + personnels dans le prompt LLM
- Diff viewer dashboard (vue Revue)
- Routes TCP sozu : SSH et PostgreSQL
- CLI `koda connect <uid>` (tunnel SSH)
- Support `devcontainer.json`
- Sélecteur de thèmes dans web-client

---

## [0.4.0] — Complété · Phase 3

### Added
- CiCdPipeline : build, lint, secret_scan, sast, dependency_scan, image_scan
- AutomationTrigger : on_push, schedule, manual
- IncomingWebhookEvent : stockage + HMAC-SHA256
- Branches éphémères pipeline (git2)
- **Sécurité intégrée** : SecurityReport, VulnerabilityFinding, SecurityPolicy
- `secret_scan` : détection credentials (regex + entropie Shannon)
- `sast` : LLM sécurité dédié (OWASP Top 10 par langage, severity scoring Critical/High/Medium/Low/Info)
- `dependency_scan` : cargo audit, npm audit, pip-audit
- `image_scan` : Trivy/Grype sur images workspace
- Dashboard : rapport sécurité + findings par workspace + blocage Revue si politique atteinte
- Dashboard : panneau pipelines + historique
- Webhook Inbox par workspace
- Pipeline IA : review automatique de diff
- Workspace Activity Feed
- Dead-letter stream jobs

---

## [1.0.0] — En cours · MVP Stable

### Added
- RBAC complet (owner, admin, developer, viewer)
- AuditEvent : traçabilité complète — `admin_audit_logs` API + `AuditEvent` tracé sur impersonation
- RLS PostgreSQL tables critiques
- TOTP MFA : setup, verify, status, delete (`/api/v1/user/mfa/*`)
- Tokens M2M avec rotation (`/api/v1/organizations/:org_id/tokens`)
- OrganizationQuota
- OpenTelemetry + Sentry
- **Rate limiting par IP + par utilisateur** : `rate_limit_middleware` global, Redis sliding window ✓
- **Tests E2E Playwright** : auth setup, workspace lifecycle, MFA, rate limiting, security headers ✓
- Garbage collector volumes orphelins
- Pre-warming images Docker
- **Snapshot workspace** : `POST|GET /api/v1/.../snapshots` ✓
- **Documentation OpenAPI générée et publiée** : `/swagger-ui` + `/api-docs/openapi.json` ✓
- **Panel admin complet** : orgs, users, impersonation, audit logs, infra stats ✓

### Security
- Review OWASP Top 10 complète
- Couverture tests ≥ 75% global, ≥ 90% sécurité/routage
