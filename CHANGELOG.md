# CHANGELOG — Koda

> Format : [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/)
> Versioning : [SemVer](https://semver.org/lang/fr/)

---

## [Unreleased]

### Added
- Analyse de faisabilité initiale (`docs/FEASIBILITY_ANALYSIS.md`)
- Décisions architecturales : Rust (Axum + SQLx), sozu gateway, Harness CI/CD
- Système de thèmes évolutif : `ThemeRegistry` observable, `SkinManifest` avec héritage `extends`, `loadFromUrl()` marketplace
- 4 skins built-in (default, minimal, pro, light) avec layouts distincts
- `ThemeSwitcher` avec miniatures `SkinPreview` générées depuis les tokens CSS
- MCP connecteurs plugins : `@koda/mcp-connectors` (registre TypeScript + 6 connecteurs built-in)
- `MCPConnectorRegistry` observable (même pattern que ThemeRegistry)
- `services/mcp-gateway/` : service Rust consommant Redis Streams `jobs:mcp`, 6 connecteurs built-in
- `McpConnector` trait Rust + `ConnectorRegistry` auto-enregistrement
- Spécification d'implémentation par module (`docs/IMPLEMENTATION_SPEC.md`)
- Roadmap versionnée v0.1.0 → v1.0.0

---

## [0.1.0] — À venir · Phase 0

### Added
- Monorepo Rust multi-crates (api, orchestrator, worker, git-manager, gateway)
- API Axum : authentification (session cookie, OAuth Google/GitHub)
- PostgreSQL + sqlx-migrate : Organization, User, Membership
- Trait `AiProviderAdapter` + implémentation Anthropic
- sozu en Docker Compose dev
- Service gateway/ : client sozu-command-lib
- Dashboard Next.js : skeleton + login
- ThemeProvider + sélecteur de thèmes (4 skins)
- Pipeline Harness : CI toutes branches + déploiement prod auto sur main

---

## [0.2.0] — À venir · Phase 1

### Added
- Workspace : création, statut, UID immuable
- WorkspaceGitConfig : clone asynchrone (pending → cloning → ready | failed)
- WorkspaceVolume : cycle de vie (création, montage, détachement)
- Container Docker via bollard + docker-socket-proxy (resource limits obligatoires)
- ExposureRule HTTP via sozu
- SSE : `/api/v1/workspaces/:uid/events`
- Dashboard : liste workspaces + création + statut temps réel

---

## [0.3.0] — À venir · Phase 2

### Added
- PluginDefinition catalogue (koda-web-ide, code-server, ssh, jupyter)
- WorkspacePluginBinding + health probe par plugin
- Koda Web IDE : Monaco Editor + file tree + terminal xterm.js + chat IA sidebar + git panel
- Endpoints API fichiers : `GET|PUT /api/v1/workspaces/:uid/files/*`
- Endpoint chat IA : `POST /api/v1/workspaces/:uid/ai/chat` (SSE streaming)
- MCP connecteurs dans le web-client : panel activation, config, statut
- MCPConnectorDefinition + WorkspaceMCPBinding (DB + API)
- mcp-gateway Rust opérationnel (6 connecteurs : github, jira, notion, postgres, slack, http)
- Injection tool definitions MCP dans le prompt LLM
- Diff viewer dashboard (vue Revue)
- Routes TCP sozu : SSH et PostgreSQL
- CLI `koda connect <uid>` (tunnel SSH)
- Support `devcontainer.json`
- Sélecteur de thèmes dans web-client

---

## [0.4.0] — À venir · Phase 3

### Added
- CiCdPipeline : build, lint, security_scan
- AutomationTrigger : on_push, schedule, manual
- IncomingWebhookEvent : stockage + HMAC-SHA256
- Branches éphémères pipeline (git2)
- Dashboard : panneau pipelines + historique
- Webhook Inbox par workspace
- Pipeline IA : review automatique de diff
- Workspace Activity Feed
- Dead-letter stream jobs

---

## [1.0.0] — À venir · MVP Stable

### Added
- RBAC complet (owner, admin, developer, viewer)
- AuditEvent : traçabilité complète
- RLS PostgreSQL tables critiques
- TOTP MFA + tokens M2M rotation
- OrganizationQuota
- OpenTelemetry + Sentry
- Rate limiting (tower middleware)
- Tests E2E Playwright (parcours critiques)
- Garbage collector volumes orphelins
- Pre-warming images Docker
- Snapshot workspace
- Documentation OpenAPI publiée

### Security
- Review OWASP Top 10 complète
- Couverture tests ≥ 75% global, ≥ 90% sécurité/routage
