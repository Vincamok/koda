# issues.md — Koda

> Tracker des issues actives. Source de vérité pour les agents IA et contributeurs.
> Références : [ROADMAP.md](ROADMAP.md) · [MEMORY.md](MEMORY.md) · [AGENTS.md](AGENTS.md)

## Légende

| Champ | Valeurs |
|-------|---------|
| Status | `open` · `in-progress` · `blocked` · `closed` |
| Priorité | `critical` · `high` · `medium` · `low` |
| Type | `feature` · `bug` · `chore` · `security` · `docs` |
| Phase | `p0` v0.1.0 · `p1` v0.2.0 · `p2` v0.3.0 · `p3` v0.4.0 · `p4` v1.0.0 |

---

## Phase 0 — Fondations (v0.1.0)

| ID | Type | Titre | Priorité | Status | Dépendances |
|----|------|-------|----------|--------|-------------|
| KODA-001 | chore | Initialisation monorepo (`apps/`, `services/`, `packages/`, `infra/`, `docs/`) | critical | open | — |
| KODA-002 | chore | Workspace Cargo multi-crates (api, orchestrator, worker, git-manager, gateway, mcp-gateway) | critical | open | KODA-001 |
| KODA-003 | chore | PostgreSQL + docker-compose dev + sqlx-migrate setup | critical | open | KODA-001 |
| KODA-004 | chore | Config par service : `config/default.yaml` + `.env.example` + figment dans chaque service | high | open | KODA-002 |
| KODA-005 | feature | API Axum : `/api/v1/auth/*` — inscription, login, logout, `GET /me` | critical | open | KODA-002 KODA-003 |
| KODA-006 | feature | Sessions cookie HttpOnly + SameSite=Strict (tower-sessions) | critical | open | KODA-005 |
| KODA-007 | feature | OAuth Google + GitHub (échange code → token, upsert user) | high | open | KODA-005 |
| KODA-008 | feature | Migrations initiales : `organizations`, `users`, `memberships` | critical | open | KODA-003 |
| KODA-009 | feature | Teams : migrations `teams`, `team_memberships`, `team_project_access`, `team_quotas` | high | open | KODA-008 |
| KODA-010 | feature | API Teams : CRUD teams + membres + accès projets + quotas | high | open | KODA-009 |
| KODA-011 | feature | PersonalSpace : migration + `koda-personal-<user-uid>` volume Docker (fondations) | high | open | KODA-008 |
| KODA-012 | chore | `TRUSTED_PROXY_CIDRS` + `axum-client-ip` sur API | high | open | KODA-005 |
| KODA-013 | feature | Trait `AiProviderAdapter` + implémentation Anthropic (reqwest + SSE) | high | open | KODA-002 |
| KODA-014 | chore | sozu en Docker Compose dev + route de test | critical | open | KODA-001 |
| KODA-015 | feature | Service `gateway/` : client sozu-command-lib — add/remove HttpFrontend + TcpFrontend | critical | open | KODA-014 |
| KODA-016 | feature | Dashboard Next.js : skeleton + page login + layout de base (responsive mobile-first) | high | open | KODA-001 |
| KODA-017 | feature | Système de thèmes : ThemeProvider + ThemeRegistry + 4 skins built-in | high | open | KODA-016 |
| KODA-018 | chore | Pipeline Harness : lint → test → build image → push registry (toutes branches) | critical | open | KODA-002 |
| KODA-019 | chore | Pipeline Harness prod : déploiement auto sur merge `main` | critical | open | KODA-018 |
| KODA-020 | chore | `koda-rust-base` image builder partagée (multi-stage, distroless runtime) | medium | open | KODA-018 |

---

## Phase 1 — Workspace minimal (v0.2.0)

| ID | Type | Titre | Priorité | Status | Dépendances |
|----|------|-------|----------|--------|-------------|
| KODA-021 | feature | Migrations : `projects`, `templates`, `workspaces`, `workspace_git_configs`, `workspace_volumes` | critical | open | KODA-008 |
| KODA-022 | feature | API : `POST /workspaces`, `GET /workspaces/:uid`, `GET /workspaces` (cursor-based) | critical | open | KODA-021 |
| KODA-023 | feature | UID immuable UUID v4 généré à la création workspace | critical | open | KODA-022 |
| KODA-024 | feature | Worker Rust : clone Git asynchrone via Redis Streams `jobs:workspace` + git2 | critical | open | KODA-002 KODA-021 |
| KODA-025 | feature | Machine d'états clone : `pending → cloning → ready | failed` | critical | open | KODA-024 |
| KODA-026 | feature | Orchestrateur : lancement container via bollard + docker-socket-proxy | critical | open | KODA-002 |
| KODA-027 | feature | HostConfig obligatoire : `cpu_period`, `cpu_quota`, `memory`, `pids_limit` | critical | open | KODA-026 |
| KODA-028 | feature | Réseaux Docker multi par workspace : `koda-ws-<uid>-internal` + `koda-ws-<uid>-services` + `koda-egress` | high | open | KODA-026 |
| KODA-029 | chore | Labels `koda.*` obligatoires sur tous les containers éphémères | critical | open | KODA-026 |
| KODA-030 | feature | Nommage container `koda-<binding-uid>` + alias réseau `svc-<binding-uid>` | high | open | KODA-026 KODA-028 |
| KODA-031 | feature | `ExposureRule` HTTP via sozu après démarrage container | critical | open | KODA-015 KODA-026 |
| KODA-032 | feature | SSE : `GET /workspaces/:uid/events` — transitions statut + logs | high | open | KODA-022 |
| KODA-033 | feature | Migration `workspace_shares` + API WorkspaceShare (editor|reviewer|viewer, expiration) | high | open | KODA-021 |
| KODA-034 | feature | Dashboard : liste workspaces + statut temps réel (EventSource) | high | open | KODA-016 KODA-032 |
| KODA-035 | feature | Dashboard : formulaire création workspace (projet + template + git URL) | high | open | KODA-034 |
| KODA-036 | feature | Dashboard multi-device : breakpoints responsive, actions rapides mobile (start/stop) | medium | open | KODA-016 |
| KODA-037 | feature | `WorkspaceVolume` : création, montage, détachement, archive | high | open | KODA-026 |
| KODA-038 | chore | Sécurité runtime containers : user non-root (uid 1000), `no-new-privileges`, seccomp profile | critical | open | KODA-026 |
| KODA-039 | chore | Images workspace pré-buildées : `ubuntu-base`, `ubuntu-node`, `ubuntu-python`, `ubuntu-go`, `ubuntu-rust` | high | open | KODA-020 |

---

## Phase 2 — Workspace complet (v0.3.0)

| ID | Type | Titre | Priorité | Status | Dépendances |
|----|------|-------|----------|--------|-------------|
| KODA-040 | feature | Migrations : `plugin_definitions`, `workspace_plugin_bindings`, `exposure_rules` | critical | open | KODA-021 |
| KODA-041 | feature | Catalogue plugins : `koda-web-ide`, `code-server`, `ssh`, `jupyter` + `PluginDefinition.network_policy` | critical | open | KODA-040 |
| KODA-042 | feature | Health probe par plugin : polling `/healthz` jusqu'à ready → statut `running` | high | open | KODA-041 |
| KODA-043 | feature | koda-web-ide : Monaco Editor (`@monaco-editor/react`, `publicPath: 'auto'`) | critical | open | KODA-001 |
| KODA-044 | feature | koda-web-ide : FileTree + `GET|PUT /workspaces/:uid/files/*` | critical | open | KODA-043 |
| KODA-045 | feature | koda-web-ide : Terminal xterm.js via WebSocket sozu | critical | open | KODA-043 |
| KODA-046 | feature | koda-web-ide : Chat IA sidebar — streaming SSE + **5 niveaux de prompt** (nano→agent) | critical | open | KODA-013 KODA-043 |
| KODA-047 | feature | `ai-context.ts` : filtre secrets (`.env`, `*.key`, `*.pem`) + détection prompt injection | critical | open | KODA-046 |
| KODA-048 | feature | koda-web-ide : Git panel (diff, stage, commit, push) | high | open | KODA-043 |
| KODA-049 | feature | koda-web-ide : détection device → 3 modes (`full-ide` | `tablet-ide` | `mobile-view`) | high | open | KODA-043 |
| KODA-050 | feature | Panel `PromptLevelSelector` + `AgentConfirmDialog` (confirmation avant action niveau 5) | high | open | KODA-046 |
| KODA-051 | feature | PersonalSpace complet : `orchestrator/personal.rs` — volume, symlinks shell/git, startup.sh | high | open | KODA-011 KODA-026 |
| KODA-052 | feature | Panel "Mon espace" dans web-client : édition Monaco de tous les fichiers `.personal/` | high | open | KODA-043 KODA-051 |
| KODA-053 | feature | Fusion `ai/CLAUDE.md` personnel + workspace dans le contexte LLM | high | open | KODA-046 KODA-051 |
| KODA-054 | feature | Migrations `personal_spaces`, `personal_snippets` + API `GET|PUT /users/me/personal/*` | high | open | KODA-008 |
| KODA-055 | feature | Snippets personnels : API CRUD + disponibles dans Monaco | medium | open | KODA-054 KODA-043 |
| KODA-056 | feature | Notes workspace personnelles : `notes/workspace-notes/<uid>.md` | medium | open | KODA-054 |
| KODA-057 | feature | Migrations `mcp_connector_definitions`, `workspace_mcp_bindings`, `user_mcp_bindings` | high | open | KODA-021 |
| KODA-058 | feature | mcp-gateway opérationnel : Redis Streams consumer + 6 connecteurs built-in | high | open | KODA-057 |
| KODA-059 | feature | `SecretResolver.resolve_binding_config()` : DB + secret store réel (TODO → remplacer stub) | high | blocked | KODA-058 |
| KODA-060 | feature | API MCP : `GET /mcp/connectors`, `POST|DELETE /workspaces/:uid/mcp/bindings` | high | open | KODA-057 |
| KODA-061 | feature | `UserMCPBinding` : API `GET|POST|DELETE /users/me/mcp/bindings` | high | open | KODA-057 |
| KODA-062 | feature | Panel MCP dans web-client : activation, config, statut par connecteur | high | open | KODA-060 KODA-043 |
| KODA-063 | feature | Injection tool definitions MCP (workspace + personnels) dans le prompt LLM | high | open | KODA-046 KODA-058 |
| KODA-064 | feature | mcp-gateway : publier résultat dans Redis → SSE vers client (TODO remplacer stub session.rs) | high | blocked | KODA-058 |
| KODA-065 | feature | Diff viewer dashboard (vue Revue) | high | open | KODA-034 |
| KODA-066 | feature | Routes TCP sozu : SSH (`2200-2999`) + PostgreSQL (`5400-5499`) | high | open | KODA-015 |
| KODA-067 | feature | CLI `koda connect <uid>` (tunnel SSH via sozu TcpFrontend) | medium | open | KODA-066 |
| KODA-068 | feature | `devcontainer.json` : lecture + pré-remplissage Template/Plugin | medium | open | KODA-041 |
| KODA-069 | feature | Sélecteur de thèmes dans le web-client | medium | open | KODA-017 KODA-043 |

---

## Phase 3 — Pipelines CI/CD + Sécurité (v0.4.0)

| ID | Type | Titre | Priorité | Status | Dépendances |
|----|------|-------|----------|--------|-------------|
| KODA-070 | feature | Migrations : `cicd_pipelines`, `automation_triggers`, `incoming_webhook_events` | critical | open | KODA-021 |
| KODA-071 | feature | Worker Rust : exécution pipeline dans container isolé éphémère + branches éphémères git2 | critical | open | KODA-070 KODA-026 |
| KODA-072 | feature | Types pipeline : `build`, `lint`, `secret_scan`, `sast`, `dependency_scan`, `image_scan` | critical | open | KODA-071 |
| KODA-073 | feature | Webhook entrant : vérification HMAC-SHA256 + stockage `IncomingWebhookEvent` | high | open | KODA-070 |
| KODA-074 | feature | Triggers : `on_push`, `schedule` (cron Rust), `manual` | high | open | KODA-070 |
| KODA-075 | security | Migrations `security_policies`, `security_reports`, `vulnerability_findings`, `scan_rules` | critical | open | KODA-021 |
| KODA-076 | security | `ScanRuleEngine` : built-in rules (entropy Shannon + regex secrets communs) + OrgScanRule + WorkspaceScanRule | critical | open | KODA-075 |
| KODA-077 | security | `run_secret_scan()` : applique ScanRuleEngine → SecurityReport + VulnerabilityFinding | critical | open | KODA-076 |
| KODA-078 | security | `run_sast()` : instancie AiProviderAdapter avec SecurityAiConfig → appel LLM → parse findings JSON | critical | open | KODA-013 KODA-075 |
| KODA-079 | security | `run_dependency_scan()` : cargo audit + npm audit + pip-audit dans container isolé | high | open | KODA-071 |
| KODA-080 | security | `run_image_scan()` : Trivy/Grype, déclencheur selon `SecurityPolicy.image_scan_trigger` | high | open | KODA-075 |
| KODA-081 | security | API SecurityPolicy : `GET|PUT /orgs/:oid/security-policy` + `GET|POST|DELETE` ScanRules | high | open | KODA-075 |
| KODA-082 | security | Dashboard : rapport sécurité par workspace + blocage Revue si policy atteinte | high | open | KODA-075 KODA-034 |
| KODA-083 | feature | Pipeline IA : review automatique de diff avant étape Revue | high | open | KODA-013 KODA-071 |
| KODA-084 | feature | Dashboard : panneau pipelines + historique exécutions | high | open | KODA-034 KODA-071 |
| KODA-085 | feature | Dashboard : Webhook Inbox par workspace | medium | open | KODA-073 KODA-034 |
| KODA-086 | feature | Workspace Activity Feed (dashboard) | medium | open | KODA-034 |
| KODA-087 | chore | Dead-letter stream : jobs échoués après 3 tentatives → `jobs:dead_letter` | high | open | KODA-024 |

---

## Phase 4 — MVP Stable (v1.0.0)

| ID | Type | Titre | Priorité | Status | Dépendances |
|----|------|-------|----------|--------|-------------|
| KODA-088 | feature | RBAC complet : enforcement Teams (lead|developer|reviewer|viewer) sur toutes les routes | critical | open | KODA-010 |
| KODA-089 | feature | `AuditEvent` : toutes les actions critiques tracées (auth, workspace, sécurité, RBAC) | critical | open | KODA-008 |
| KODA-090 | security | RLS PostgreSQL sur tables critiques (workspaces, pipelines, security_reports...) | critical | open | KODA-003 |
| KODA-091 | security | TOTP MFA (totp-rs) + tokens M2M avec rotation (RFC 7009) | high | open | KODA-005 |
| KODA-092 | feature | `OrganizationQuota` : champs `max_workspaces`, `max_cpu_cores`, `max_ram_gb`, `max_storage_gb`, `max_members` + enforcement à la création workspace | high | open | KODA-022 |
| KODA-093 | chore | OpenTelemetry export OTLP + intégration Sentry sur tous les services Rust | high | open | KODA-002 |
| KODA-094 | chore | Rate limiting par IP + par user (tower middleware) | high | open | KODA-012 |
| KODA-095 | chore | Tests E2E Playwright : création workspace, revue diff, clôture | high | open | — |
| KODA-096 | chore | Couverture tests ≥ 75% global, ≥ 90% modules sécurité/routage | high | open | — |
| KODA-097 | security | Review sécurité OWASP Top 10 complète | critical | open | — |
| KODA-098 | chore | Garbage collector volumes orphelins (worker cron — labels `koda.*`) | high | open | KODA-029 KODA-037 |
| KODA-099 | chore | Pre-warming images Docker (worker cron quotidien) | medium | open | KODA-039 |
| KODA-100 | feature | Snapshot workspace (docker pause + copie volume) | medium | open | KODA-037 |
| KODA-101 | docs | Documentation OpenAPI générée et publiée | medium | open | — |
| KODA-102 | feature | Migration `super_admin` + bootstrap via `BOOTSTRAP_SUPER_ADMIN_EMAIL` | critical | open | KODA-008 |
| KODA-103 | feature | `apps/admin/` : skeleton Next.js + layout + auth `super_admin` | high | open | KODA-102 |
| KODA-104 | feature | Panel admin : tableau de bord global (métriques temps réel, santé services) | high | open | KODA-103 KODA-093 |
| KODA-105 | feature | Panel admin : gestion organisations (CRUD, quotas, suspension, impersonation) | high | open | KODA-103 KODA-092 KODA-089 |
| KODA-106 | feature | Panel admin : gestion utilisateurs (vue globale, désactivation, reset MFA, impersonation) | high | open | KODA-103 KODA-089 |
| KODA-107 | feature | Panel admin : IA & pré-prompts (provider global, system prompt par org, templates niveaux éditables) | high | open | KODA-103 KODA-013 |
| KODA-108 | feature | Panel admin : logs & audit (vue `AuditEvent` filtrée, jobs Redis dead-letter, export CSV/JSON) | high | open | KODA-103 KODA-089 |
| KODA-109 | feature | Panel admin : infrastructure (containers `koda.managed`, routes sozu, taille DB, GC manuel) | medium | open | KODA-103 |
| KODA-110 | feature | Panel admin : sécurité (ScanRule built-in, SecurityPolicy par org, derniers rapports) | medium | open | KODA-103 KODA-081 |
| KODA-111 | feature | `KodaInstance` + `OrgInstanceAffinity` : migrations + API `GET /admin/instances` + endpoint health M2M | medium | open | KODA-091 KODA-102 |
| KODA-112 | feature | Panel admin : multi-instances (vue agrégée métriques, affectation org → instance) | medium | open | KODA-103 KODA-111 |
| KODA-113 | feature | Migration `secret_refs` : table chiffrée AES-256-GCM, clé dans env | critical | open | KODA-008 |
| KODA-114 | feature | OAuth Authentik : provider OIDC générique (extensible pour autres IdP) | high | open | KODA-007 |
| KODA-115 | feature | `packages/shared-types/` : types TS partagés entre dashboard, web-client et admin | medium | open | KODA-001 |
| KODA-116 | feature | `packages/api-client/` : client HTTP TypeScript généré depuis l'OpenAPI Koda | medium | open | KODA-101 KODA-115 |

---

## Backlog post-v1.0.0

| ID | Type | Titre | Note |
|----|------|-------|------|
| KODA-B01 | feature | Workspace forking | Clone d'un workspace existant |
| KODA-B02 | feature | Terminaux partagés (pair programming) | WebRTC ou multiplexage PTY |
| KODA-B03 | feature | Env Manager UI (variables d'env par workspace) | |
| KODA-B04 | feature | Marketplace thèmes (`themeRegistry.loadFromUrl()`) | |
| KODA-B05 | feature | Connecteurs MCP community (stdio, marketplace) | `proxy.rs` activé |
| KODA-B06 | feature | Support multi-runtime dans les templates (matrix) | |
| KODA-B07 | feature | Auto-hibernation workspaces inactifs | Détection idle + suspend |
| KODA-B08 | feature | Centralisation logs avec recherche (Loki) | |
| KODA-B09 | feature | Alerting crash boucle + surconsommation | |
| KODA-B10 | chore | Support multi-VPS (RemoteDockerClient derrière le trait DockerClient) | |
| KODA-B11 | feature | Shadow deployment pour comparaison | |
| KODA-B12 | feature | Export artefacts vers S3-compatible | |
| KODA-B13 | feature | Pipeline IA refactoring/cleanup automatique | |
| KODA-B14 | feature | Multi-instances avancé : migration d'org d'une instance à l'autre, load balancing | `KodaInstance` étendu |
| KODA-B15 | feature | `TicketRecord` : lien workspace ↔ ticket externe (Jira, Linear, GitHub Issues) | Dépend connecteurs MCP |
| KODA-B16 | feature | Marketplace de plugins workspace | Au-delà des 4 built-in |

---

## Issues connues — bugs et dettes techniques

| ID | Type | Titre | Priorité | Status | Localisation |
|----|------|-------|----------|--------|--------------|
| KODA-D01 | bug | `SecretResolver.resolve_binding_config()` — stub, ne résout rien | critical | open | `services/mcp-gateway/src/secret.rs:19` |
| KODA-D02 | bug | mcp-gateway — résultat tool call non publié dans Redis (TODO dans session.rs) | critical | open | `services/mcp-gateway/src/session.rs:71` |
| KODA-D03 | chore | `PostgresConnector` — pool SQLx créé à chaque call (TODO connection pooling) | medium | open | `services/mcp-gateway/src/connectors/postgres.rs` |
| KODA-D04 | chore | `proxy.rs` — stub vide, MCP stdio non implémenté (prévu v0.4.0+) | low | open | `services/mcp-gateway/src/proxy.rs` |
