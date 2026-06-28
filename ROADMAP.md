# ROADMAP — Koda

> Versioning : SemVer — `MAJOR.MINOR.PATCH`
> - `0.x.x` : développement pré-MVP
> - `1.0.0` : MVP stable (Phase 4 complète)
> - Chaque phase = une version mineure

---

## Vue d'ensemble

```
v0.1.0  Phase 0 — Fondations          S1-4
v0.2.0  Phase 1 — Workspace minimal   S5-10
v0.3.0  Phase 2 — Workspace complet   S11-16
v0.4.0  Phase 3 — Pipelines CI/CD     S17-22
v1.0.0  Phase 4 — Sécurité & Obs.     S23-26
```

---

## v0.1.0 — Fondations `[Phase 0 · S1-4]` ✓

### Objectif
Infrastructure de base opérationnelle : monorepo, API authentifiée, BDD, proxy sozu, pipeline CI.

### Livrables
- [x] Monorepo initialisé (`apps/`, `services/`, `packages/`, `infra/`, `docs/`)
- [x] Workspace Cargo multi-crates (api, orchestrator, worker, git-manager, gateway)
- [x] PostgreSQL + sqlx-migrate : modèles `Organization`, `User`, `Membership`
- [x] **Teams** : `Team`, `TeamMembership`, `TeamProjectAccess`, `TeamQuota`
- [x] API Axum : endpoints `/api/v1/auth/*` (inscription, connexion, OAuth Google/GitHub/Authentik OIDC)
- [x] Sessions cookie HttpOnly + SameSite=Strict
- [x] Trait `AiProviderAdapter` + implémentation Anthropic HTTP (reqwest)
- [x] sozu en Docker Compose dev avec route de test (`infra/sozu/sozu.toml`)
- [x] Service `gateway/` : client sozu-command-lib minimal (add/remove HttpFrontend)
- [x] Dashboard Next.js : skeleton + page login + layout de base (responsive mobile-first)
- [ ] Pipeline Harness : lint → test → build image → push registry
- [ ] Pipeline Harness prod : déploiement auto sur merge `main`
- [x] Système de thèmes : ThemeProvider + 4 skins (default, minimal, pro, light)
- [x] Config par service : `config/default.yaml` + `.env.example` dans chaque service
- [x] `figment` pour le chargement config (YAML + env + .env)
- [x] `TRUSTED_PROXY_CIDRS` + `axum-client-ip` sur l'API
- [x] `PersonalSpace` : modèle DB + volume Docker `koda-personal-<user-uid>` (fondations)
- [x] `apps/admin/` : skeleton Next.js + layout + authentification `super_admin`
- [x] Rôle `super_admin` : migration + bootstrap via `BOOTSTRAP_SUPER_ADMIN_EMAIL`
- [x] `packages/shared-types/` + `packages/api-client/` (client HTTP généré depuis OpenAPI)
- [x] `SecretRef` : modèle DB + colonne chiffrée AES-256-GCM
- [x] **i18n** : `packages/i18n/` + `next-intl` sur les 3 apps + fichiers messages FR/EN/ES/DE
- [x] `user_settings` : migration + API `GET|PUT /api/v1/user/settings` (locale, theme)

### Critères de validation
- `cargo test --workspace` passe
- Login/logout fonctionnel en local (Google, GitHub, Authentik)
- sozu route une requête de test
- Pipeline Harness vert
- Login `super_admin` → dashboard admin accessible

---

## v0.2.0 — Workspace minimal `[Phase 1 · S5-10]` ✓

### Objectif
Créer un workspace, cloner un repo, lancer un container, accéder via URL.

### Livrables
- [x] Modèles DB : `Workspace`, `WorkspaceGitConfig`, `WorkspaceVolume`, `Template`
- [x] API : `POST /api/v1/workspaces`, `GET /api/v1/workspaces/:uid`
- [x] UID immuable généré à la création (UUID v4)
- [ ] Clone Git asynchrone via worker Rust + Redis Streams (`jobs:workspace`)
- [ ] Machine d'états clone : `pending → cloning → ready | failed`
- [x] Lancement container via bollard + docker-socket-proxy
- [x] Resource limits obligatoires dans HostConfig (cpu_period, cpu_quota, memory, pids_limit)
- [x] Réseaux Docker multi par workspace : `koda-ws-<uid>-internal` + `koda-ws-<uid>-services`
- [x] Labels `koda.*` obligatoires sur tous les containers
- [x] ExposureRule HTTP créée via sozu après démarrage container
- [x] SSE : `GET /api/v1/organizations/:org_id/workspaces/:workspace_id/events` (transitions de statut)
- [x] Dashboard : liste workspaces + formulaire création
- [x] Dashboard : statut temps réel (EventSource + `WorkspaceStatusLive`)
- [x] Dashboard multi-device : responsive mobile-first, détection breakpoints
- [x] `WorkspaceVolume` : migration + montage dans HostConfig
- [x] `WorkspaceShare` : partage ad-hoc (editor|reviewer|viewer), expiration

### Critères de validation
- Workspace créé → repo cloné → container lancé → URL `/[UID]/[service]` accessible
- Statut mis à jour en temps réel dans le dashboard
- Destruction workspace → volume préservé
- Dashboard utilisable sur mobile (monitoring, start/stop)

---

## v0.3.0 — Workspace complet `[Phase 2 · S11-16]` ✓

### Objectif
Plugin binding, health probe, Web IDE natif, diff viewer.

### Livrables
- [x] Modèles DB : `PluginDefinition`, `WorkspacePluginBinding`
- [x] Catalogue plugins : `koda-web-ide`, `code-server`, `ssh`, `jupyter`
- [x] Health probe par plugin (polling `/healthz` interne jusqu'à ready) — `plugin_prober.rs`
- [x] Statut workspace `running` déclenché par health probe OK
- [x] `koda-web-ide` plugin complet :
  - [x] Monaco Editor (`apps/web-client/src/components/ide/code-editor.tsx`)
  - [x] File tree + `GET|PUT /api/v1/workspaces/:uid/files/*`
  - [ ] Terminal xterm.js via WebSocket sozu
  - [x] Chat IA sidebar — **5 niveaux de prompt** (nano, quick, standard, deep, agent)
  - [x] Filtre secrets avant envoi IA (pas de `.env`, `*.key` dans le contexte)
  - [x] Détection du device → mode `full-ide | tablet-ide | mobile-view`
  - [x] Git panel (diff, stage, commit, push)
- [x] **PersonalSpace complet** :
  - [x] Volume Docker `koda-personal-<user-uid>` monté en read-only dans chaque workspace
  - [ ] Shell configs (`~/.personal/shell/`) sourcés dans terminal xterm.js
  - [ ] Git config personnelle (`~/.personal/git/.gitconfig`) montée dans container
  - [ ] Panel "Mon espace" dans web-client : édition Monaco de tous les fichiers `.personal/`
  - [x] Fusion `ai/instructions.md` personnel + workspace `KODA.md` dans le contexte LLM (6 couches)
  - [x] `UserMCPBinding` : connecteurs MCP personnels
  - [x] Snippets personnels disponibles dans Monaco
  - [x] Notes par workspace (`notes/workspace-notes/<uid>.md`)
- [x] Diff viewer dans le dashboard (vue Revue, étape 7)
- [x] Routes TCP sozu : SSH (`2200-2999`), PostgreSQL (`5400-5499`)
- [ ] CLI `koda connect <uid>` (tunnel SSH via sozu TcpFrontend)
- [x] Sélecteur de thèmes dans le dashboard et le web-client
- [ ] `devcontainer.json` : lecture et pré-remplissage Template/Plugin
- [x] **i18n complète** : traductions exhaustives FR/EN/ES/DE sur les 3 apps
- [x] Injection langue `UserSettings.locale` en couche 6 du contexte LLM (`AiContextBuilder`)
- [x] MCP connecteurs — intégration dans le web-client :
  - [x] Modèles DB : `MCPConnectorDefinition`, `WorkspaceMCPBinding`
  - [x] `mcp-gateway` : service Rust (Redis Streams consumer, 6 connecteurs built-in)
  - [x] Connecteurs built-in : jira, notion, postgres, slack, http (+ github à compléter)
  - [x] API : `GET /api/v1/mcp/connectors`, `POST|DELETE /api/v1/workspaces/:uid/mcp/bindings`
  - [x] Panel MCP dans web-client (activation, config, statut par connecteur)
  - [ ] Injection tool definitions MCP dans le prompt LLM lors du chat IA
  - [x] SecretRef : résolution credentials au moment du tool call, jamais loggé
  - [x] `@koda/mcp-connectors` : registre TypeScript + connecteurs built-in
- [x] **Pré-prompts LLM-agnostiques** :
  - [x] Packs langue built-in (`rust`, `typescript`, `python`, `go`, `sql`) — non supprimables
  - [x] Packs framework built-in (`axum`, `react`, `nextjs`, `sqlx`) — non supprimables
  - [x] Auto-détection packs depuis manifestes repo (`Cargo.toml`, `package.json`, `next.config.*`…)
  - [x] Context builder dans `orchestrator` : assemblage 6 couches par niveau de prompt
  - [x] Support `KODA.md` à la racine du repo (couche 5, LLM-agnostique)

### Critères de validation
- Ouverture web-client → édition fichier → commit visible dans diff viewer
- Chat IA → patch proposé → appliqué en un clic
- `koda connect <uid>` établit une session SSH fonctionnelle
- Activation connecteur GitHub → le chat IA peut lister les issues du repo
- Workspace Rust → packs `rust` + `axum` auto-détectés → instructions Rust injectées dans le prompt

---

## v0.4.0 — Pipelines CI/CD `[Phase 3 · S17-22]` ✓

### Objectif
Pipelines de vérification automatisés, webhooks, triggers.

### Livrables
- [x] Modèles DB : `CiCdPipeline`, `AutomationTrigger`, `IncomingWebhookEvent`
- [x] Worker Rust : exécution pipeline dans container isolé éphémère (`pipeline_runner.rs`)
- [x] Types de pipeline : `build`, `lint`, `security_scan`, `dependency_scan`, `image_scan`, `secret_scan`
- [x] Branches éphémères pipeline : `pipeline/<uid>/<timestamp>` (git2 — fallback gracieux si repo non cloné)
- [x] Webhook entrant : vérification HMAC-SHA256 + stockage `IncomingWebhookEvent`
- [x] Triggers : `on_push`, `schedule` (cron Rust — `cron_scheduler.rs`), `manual`
- [x] API : endpoints pipelines + triggers + run
- [x] Dashboard : panneau pipelines + historique exécutions (`GET .../pipelines/{id}/runs`)
- [x] Webhook Inbox par workspace (dashboard)
- [x] **Sécurité intégrée dans les projets** :
  - [x] `SecurityReport`, `VulnerabilityFinding`, `SecurityPolicy`
  - [x] `secret_scan` : détection credentials (regex + entropie Shannon)
  - [x] `sast` : LLM Anthropic (claude-haiku) OWASP Top 10 — findings parsés + sauvegardés en DB
  - [x] `dependency_scan` : cargo audit, npm audit, pip-audit
  - [x] `image_scan` : container éphémère bollard + Trivy (resource limits, labels koda, cleanup)
  - [x] Dashboard : rapport sécurité + findings par workspace
  - [x] Blocage phase Revue si `SecurityPolicy.min_severity_to_block` atteint → workspace → `reviewing`
- [x] Pipeline IA : review automatique de diff avant étape Revue (`diff_review` pipeline — LLM Anthropic, git2, table `diff_reviews`)
- [x] Dead-letter stream : jobs échoués après 3 tentatives (`koda:jobs:pipeline:dead`)
- [x] Workspace Activity Feed (dashboard) — `GET .../workspaces/{id}/activity` sur `audit_events`

### Critères de validation
- Push Git → webhook → pipeline déclenché → résultat visible dans dashboard
- `secret_scan` détecte un token hardcodé dans le code
- `sast` produit un rapport avec severity avant la Revue
- Pipeline IA produit un résumé diff avant la revue
- `schedule` trigger s'exécute à l'heure configurée

---

## v1.0.0 — MVP Stable `[Phase 4 · S23-26]` — En cours

### Objectif
Sécurité renforcée, observabilité, tests E2E, audit.

### Livrables
- [ ] RBAC complet : Teams + rôles (lead | developer | reviewer | viewer) + WorkspaceShare
- [ ] `SecurityPolicy` org-level : audit des scans requis + seuil de blocage configurable
- [x] `AuditEvent` : toutes les actions critiques tracées (`audit.rs` + `admin_audit_logs`)
- [x] RLS PostgreSQL sur tables critiques (`202600010035_enable_rls.sql`)
- [x] TOTP MFA (totp-rs) + tokens M2M avec rotation (RFC 7009)
- [ ] `OrganizationQuota` : limites par org (`max_workspaces`, `max_cpu_cores`, `max_ram_gb`, `max_storage_gb`, `max_members`)
- [ ] OpenTelemetry export OTLP + intégration Sentry
- [x] Rate limiting par IP + par utilisateur (tower middleware — `rate_limit.rs`)
- [x] Tests E2E Playwright : création workspace, revue diff, clôture
- [ ] Couverture tests ≥ 75% global, ≥ 90% modules sécurité/routage
- [ ] Review sécurité OWASP Top 10
- [x] Garbage collector volumes orphelins (worker cron — `garbage_collector.rs`)
- [x] Pre-warming images Docker (worker cron quotidien — `garbage_collector.rs`)
- [x] Documentation OpenAPI générée et publiée (`/swagger-ui` + `/api-docs/openapi.json`)
- [x] Snapshot workspace (docker pause + copie volume)
- [x] **Panel admin complet (`apps/admin/`)** :
  - [ ] Dashboard global : métriques temps réel, santé des services
  - [x] Gestion organisations : CRUD, quotas, suspension (`admin_toggle_org`)
  - [x] Gestion utilisateurs : global, impersonation, reset MFA
  - [ ] IA & pré-prompts : provider global, system prompt par org, templates de niveaux
  - [x] Logs & audit : vue unifiée `AuditEvent` + jobs Redis + export CSV/JSON
  - [x] Infrastructure : containers actifs, routes sozu, DB, GC manuel (`admin_infra_stats`)
  - [x] Endpoint `GET /api/v1/admin/health` (authentifié M2M) pour multi-instances
- [ ] `KodaInstance` + `OrgInstanceAffinity` : fondations multi-instances

### Critères de validation
- Parcours E2E complets verts
- Aucun finding critique OWASP
- Toutes les actions critiques présentes dans `audit_events`
- Super admin : accès complet au panel admin, impersonation tracée
- Quotas org : dépassement bloque la création workspace

---

## Backlog post-v1.0.0

- Workspace forking
- Env Manager UI (variables d'environnement par workspace)
- Terminaux partagés (pair programming WebRTC ou multiplexage PTY)
- Shadow deployment pour comparaison
- Support multi-runtime (Node/Python/Go) dans les templates (matrix)
- Auto-hibernation des workspaces inactifs (détection idle + suspend)
- Centralisation des logs avec recherche (Loki)
- Alerting sur crash boucle et surconsommation
- Pipeline IA refactoring/cleanup automatique
- Export artefacts vers S3-compatible
- Connecteurs MCP community (stdio, marketplace) — `proxy.rs` activé
- `themeRegistry.loadFromUrl()` marketplace de thèmes
- **Multi-instances avancé** : migration d'org d'une instance à l'autre, load balancing inter-instances
- **`TicketRecord`** : lien workspace ↔ ticket externe (Jira, Linear, GitHub Issues)
- **Marketplace de plugins** workspace (au-delà des 4 built-in)
