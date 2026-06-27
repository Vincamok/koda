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

## [0.4.0] — À venir · Phase 3

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
