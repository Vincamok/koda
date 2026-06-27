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

## v0.1.0 — Fondations `[Phase 0 · S1-4]`

### Objectif
Infrastructure de base opérationnelle : monorepo, API authentifiée, BDD, proxy sozu, pipeline CI.

### Livrables
- [ ] Monorepo initialisé (`apps/`, `services/`, `packages/`, `infra/`, `docs/`)
- [ ] Workspace Cargo multi-crates (api, orchestrator, worker, git-manager, gateway)
- [ ] PostgreSQL + sqlx-migrate : modèles `Organization`, `User`, `Membership`
- [ ] API Axum : endpoints `/api/v1/auth/*` (inscription, connexion, OAuth Google/GitHub)
- [ ] Sessions cookie HttpOnly + SameSite=Strict
- [ ] Trait `AiProviderAdapter` + implémentation Anthropic HTTP (reqwest)
- [ ] sozu en Docker Compose dev avec route de test
- [ ] Service `gateway/` : client sozu-command-lib minimal (add/remove HttpFrontend)
- [ ] Dashboard Next.js : skeleton + page login + layout de base
- [ ] Pipeline Harness : lint → test → build image → push registry
- [ ] Pipeline Harness prod : déploiement auto sur merge `main`
- [ ] Système de thèmes : ThemeProvider + 4 skins (default, minimal, pro, light)

### Critères de validation
- `cargo test --workspace` passe
- Login/logout fonctionnel en local
- sozu route une requête de test
- Pipeline Harness vert

---

## v0.2.0 — Workspace minimal `[Phase 1 · S5-10]`

### Objectif
Créer un workspace, cloner un repo, lancer un container, accéder via URL.

### Livrables
- [ ] Modèles DB : `Workspace`, `WorkspaceGitConfig`, `WorkspaceVolume`, `Template`
- [ ] API : `POST /api/v1/workspaces`, `GET /api/v1/workspaces/:uid`
- [ ] UID immuable généré à la création (UUID v4)
- [ ] Clone Git asynchrone via worker Rust + Redis Streams (`jobs:workspace`)
- [ ] Machine d'états clone : `pending → cloning → ready | failed`
- [ ] Lancement container via bollard + docker-socket-proxy
- [ ] Resource limits obligatoires dans HostConfig (CPU, RAM, PID)
- [ ] ExposureRule HTTP créée via sozu après démarrage container
- [ ] SSE : `GET /api/v1/workspaces/:uid/events` (transitions de statut)
- [ ] Dashboard : liste workspaces + statut temps réel (EventSource)
- [ ] Dashboard : formulaire création workspace (projet + template + git URL)
- [ ] `WorkspaceVolume` : création, montage, détachement

### Critères de validation
- Workspace créé → repo cloné → container lancé → URL `/[UID]/[service]` accessible
- Statut mis à jour en temps réel dans le dashboard
- Destruction workspace → volume préservé

---

## v0.3.0 — Workspace complet `[Phase 2 · S11-16]`

### Objectif
Plugin binding, health probe, Web IDE natif, diff viewer.

### Livrables
- [ ] Modèles DB : `PluginDefinition`, `WorkspacePluginBinding`
- [ ] Catalogue plugins : `koda-web-ide`, `code-server`, `ssh`, `jupyter`
- [ ] Health probe par plugin (polling `/healthz` interne jusqu'à ready)
- [ ] Statut workspace `running` déclenché par health probe OK
- [ ] `koda-web-ide` plugin complet :
  - [ ] Monaco Editor avec `publicPath: 'auto'` (Vite)
  - [ ] File tree + `GET|PUT /api/v1/workspaces/:uid/files/*`
  - [ ] Terminal xterm.js via WebSocket sozu
  - [ ] Chat IA sidebar (SSE streaming via AiProviderAdapter)
  - [ ] Git panel (diff, stage, commit, push)
- [ ] Diff viewer dans le dashboard (vue Revue, étape 7)
- [ ] Routes TCP sozu : SSH (`2200-2999`), PostgreSQL (`5400-5499`)
- [ ] CLI `koda connect <uid>` (tunnel SSH via sozu TcpFrontend)
- [ ] Sélecteur de thèmes dans le dashboard et le web-client
- [ ] `devcontainer.json` : lecture et pré-remplissage Template/Plugin

### Critères de validation
- Ouverture web-client → édition fichier → commit visible dans diff viewer
- Chat IA → patch proposé → appliqué en un clic
- `koda connect <uid>` établit une session SSH fonctionnelle

---

## v0.4.0 — Pipelines CI/CD `[Phase 3 · S17-22]`

### Objectif
Pipelines de vérification automatisés, webhooks, triggers.

### Livrables
- [ ] Modèles DB : `CiCdPipeline`, `AutomationTrigger`, `IncomingWebhookEvent`
- [ ] Worker Rust : exécution pipeline dans container isolé éphémère
- [ ] Types de pipeline : `build`, `lint`, `security_scan`
- [ ] Branches éphémères pipeline : `pipeline/<uid>/<timestamp>` (git2)
- [ ] Webhook entrant : vérification HMAC-SHA256 + stockage `IncomingWebhookEvent`
- [ ] Triggers : `on_push`, `schedule` (cron Rust), `manual`
- [ ] API : endpoints pipelines + triggers + run
- [ ] Dashboard : panneau pipelines + historique exécutions
- [ ] Webhook Inbox par workspace (dashboard)
- [ ] Pipeline IA : review automatique de diff avant étape Revue
- [ ] Dead-letter stream : jobs échoués après 3 tentatives
- [ ] Workspace Activity Feed (dashboard)

### Critères de validation
- Push Git → webhook → pipeline déclenché → résultat visible dans dashboard
- Pipeline IA produit un résumé diff avant la revue
- `schedule` trigger s'exécute à l'heure configurée

---

## v1.0.0 — MVP Stable `[Phase 4 · S23-26]`

### Objectif
Sécurité renforcée, observabilité, tests E2E, audit.

### Livrables
- [ ] RBAC complet : `owner`, `admin`, `developer`, `viewer`
- [ ] `AuditEvent` : toutes les actions critiques tracées
- [ ] RLS PostgreSQL sur tables critiques
- [ ] TOTP MFA (totp-rs) + tokens M2M avec rotation (RFC 7009)
- [ ] `OrganizationQuota` : quotas appliqués à la création workspace
- [ ] OpenTelemetry export OTLP + intégration Sentry
- [ ] Rate limiting par IP + par utilisateur (tower middleware)
- [ ] Tests E2E Playwright : création workspace, revue diff, clôture
- [ ] Couverture tests ≥ 75% global, ≥ 90% modules sécurité/routage
- [ ] Review sécurité OWASP Top 10
- [ ] Garbage collector volumes orphelins (worker cron)
- [ ] Pre-warming images Docker (worker cron quotidien)
- [ ] Documentation OpenAPI générée et publiée
- [ ] Snapshot workspace (docker pause + copie volume)

### Critères de validation
- Parcours E2E complets verts
- Aucun finding critique OWASP
- Toutes les actions critiques présentes dans `audit_events`

---

## Backlog post-v1.0.0

- Workspace forking
- Env Manager UI (variables d'environnement)
- Terminaux partagés (pair programming)
- `OrganizationQuota` UI admin
- Shadow deployment pour comparaison
- Support multi-runtime (Node/Python/Go) dans les templates
- Auto-hibernation des workspaces inactifs
- Centralisation des logs avec recherche (Loki)
- Alerting sur crash boucle et surconsommation
- Pipeline IA refactoring/cleanup
- Pipeline IA hardening sécurité
- Export artefacts vers S3-compatible
