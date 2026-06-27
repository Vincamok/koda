# MEMORY.md — Koda

## Contexte projet
Koda est une plateforme de gestion d'environnements de développement à la demande, auto-hébergée sur VPS. Chaque workspace est un conteneur Docker isolé, accessible via une URL unique `domain.com/[UID]/[service]` gérée par **sozu** (reverse-proxy Rust avec API de configuration dynamique).

## Décisions architecturales actées

| Décision | Choix retenu | Alternative écartée |
|----------|-------------|---------------------|
| Gateway | **sozu** (reverse-proxy Rust, TLS + HTTP/2 + TCP natifs) | nginxify, Traefik, proxy custom |
| Routing HTTP | Path-based `[UID]/[service]` via sozu HttpFrontend | — |
| Routing TCP | Port-based via sozu TcpFrontend (SSH, PostgreSQL) | HAProxy |
| Port forwarding HTTP | sozu HttpFrontend + StripPrefix | — |
| Port forwarding TCP | sozu TcpFrontend (port dédié par tunnel) | — |
| Backend API | **Axum** (Rust) + SQLx + tokio | FastAPI (Python) |
| Workers / Task Runner | **Rust** (tokio + Redis Streams) | Celery (Python) |
| Docker SDK (Rust) | **bollard** (Docker API async) | docker-py |
| Git (Rust) | **git2** (libgit2 bindings) | shell git |
| LLM integration | **reqwest** + HTTP direct (AiProviderAdapter trait) | SDK Python |
| Frontend dashboard | **Next.js** + TypeScript + shadcn/ui + Tailwind | SvelteKit |
| Web IDE client | **Monaco Editor** + xterm.js + chat IA sidebar | code-server seul |
| BDD | PostgreSQL | — |
| Migrations | **sqlx-migrate** (fichiers SQL versionnés) | Alembic |
| Isolation socket Docker | docker-socket-proxy (whitelist API) | Socket brut |
| Queue broker | Redis Streams (consumer groups) | RabbitMQ |
| CI/CD | **Harness self-hosted** (mirror GitHub) | GitHub Actions |
| Registry images | **Harness Artifact Registry** | Docker Hub, GHCR |
| Déploiement prod | Auto sur merge `main` via Harness pipeline | Manuel |

## Environnements

### Dev
- Code sur **GitHub** (`main` + branches `feature/*`, `fix/*`)
- Mirror automatique GitHub → **Harness self-hosted** (effet miroir)
- Harness build les images Docker sur chaque push/PR
- Images taguées `sha-<commit>` et poussées vers le **Harness registry**
- 2 développeurs, pas de client externe, pas de staging séparé

### Prod
- Déploiement déclenché automatiquement par Harness sur merge `main`
- Pull des images depuis le Harness registry (tag `sha-<commit>` du merge)
- Rollback = redéployer le tag précédent depuis Harness

### Pipeline CI (Harness) — toutes branches
1. `cargo test --workspace` + `cargo clippy`
2. `npm run test` + `npm run lint` (dashboard + web-client)
3. Build images Docker multi-stage
4. Push vers Harness registry (tag `sha-<commit>`)
5. **Sur merge `main` uniquement :** déploiement prod automatique

## Plugin catalogue

| Plugin | Type | Description |
|--------|------|-------------|
| `koda-web-ide` | web | **Client web natif Koda** — Monaco Editor + file tree + terminal xterm.js + chat IA sidebar. Endpoint `/[UID]/ide` |
| `code-server` | web | VS Code complet dans le navigateur. Endpoint `/[UID]/vscode` |
| `ssh` | tcp | Accès SSH direct via tunnel sozu TcpFrontend (port dédié) |
| `jupyter` | web | JupyterLab pour workspaces data/ML. Endpoint `/[UID]/jupyter` |

## Koda Web IDE (`koda-web-ide`)

Client web natif, différenciateur de la plateforme. Vit dans `apps/web-client/`.

**Composants :**
- **Monaco Editor** (Apache 2.0) — moteur VS Code, support syntaxe, LSP
- **File tree** — lecture/écriture fichiers via `GET|PUT /api/v1/workspaces/:uid/files`
- **Terminal** — xterm.js + WebSocket sozu → container PTY
- **Chat IA sidebar** — streaming SSE via `POST /api/v1/workspaces/:uid/ai/chat`
  - Lit les fichiers ouverts comme contexte
  - Propose des patches appliqués en un clic
  - Utilise `AiProviderAdapter` (Anthropic par défaut)
- **Git panel** — diff, stage, commit, push via Koda API

**Nouveaux endpoints API requis :**
```
GET  /api/v1/workspaces/:uid/files?path=/src     # Liste le répertoire
GET  /api/v1/workspaces/:uid/files/content?path= # Lit un fichier
PUT  /api/v1/workspaces/:uid/files/content?path= # Écrit un fichier
POST /api/v1/workspaces/:uid/ai/chat             # Chat IA (SSE streaming)
GET  /api/v1/workspaces/:uid/ai/chat/:id         # Historique session
```

## Intégration sozu

Le service `services/gateway/` est un client Rust de sozu via `sozu-command-lib`.  
Il traduit les `ExposureRule` Koda en commandes sozu, sans jamais éditer de config fichier.

**Routes HTTP :**
```
[UID]/ide     → HttpFrontend { path_prefix: "/[UID]/ide",    strip_prefix: true } → container:4000
[UID]/vscode  → HttpFrontend { path_prefix: "/[UID]/vscode", strip_prefix: true } → container:8080
```

**Routes TCP (port dédié par tunnel) :**
```
:2201 → TcpFrontend → container SSH :22
:5433 → TcpFrontend → container PostgreSQL :5432
```
Plages réservées : SSH `2200-2999`, PostgreSQL `5400-5499`. Stockées dans `ExposureRule.host_port`.

## Entités métier clés
- `Workspace` : instance identifiée par UID immuable. Statuts : `created → configuring → running → reviewing → closing → closed`
- `WorkspaceGitConfig` : config Git du workspace (1 actif max), clone_status : `pending → cloning → ready | failed`
- `WorkspacePluginBinding` : plugin actif déclenchant le provisioning container. Statuts : `installing → ready | failed`
- `ExposureRule` : mapping route ↔ container. Champs : `protocol (http|tcp)`, `public_path | host_port`, `internal_host`, `internal_port`, `strip_prefix`
- `WorkspaceVolume` : volume Docker persistant lié au workspace
- `CiCdPipeline` : pipeline build/lint/security. Statuts : `idle → running → passed | failed`
- `AutomationTrigger` : déclencheur on_push | schedule | manual
- `IncomingWebhookEvent` : event webhook entrant stocké (TTL 7j) avant traitement worker
- `OrganizationQuota` : limites de ressources par organisation
- `AuditEvent` : traçabilité de toutes les actions critiques

## Contraintes non-négociables
- Pas de Docker-in-Docker (DinD)
- Path Stripping HTTP obligatoire via sozu (l'app ne voit jamais le préfixe /[UID]/)
- organization_id obligatoire sur toutes les entités exposées (+ RLS PostgreSQL)
- Aucun secret stocké en clair (SecretRef uniquement)
- Limites CPU/RAM/PID obligatoires sur chaque conteneur workspace (bollard HostConfig)
- Opérations fichiers web-client : toujours via Koda API, jamais accès direct au volume

## Risques principaux
1. Socket Docker = vecteur root → mitigé par docker-socket-proxy
2. WebSocket → sozu supporte HTTP upgrade nativement
3. TLS → sozu gère la terminaison TLS + renouvellement Let's Encrypt
4. Absence de health probe par plugin → mécanisme probe défini dans PluginDefinition
5. Volumes orphelins → garbage collector planifié (worker Rust cron)
6. LLM sans abstraction → AiProviderAdapter trait dès Phase 0
7. Accès fichiers web-client non authentifié → middleware auth Axum sur tous les endpoints `/files`

## Arborescence projet
```
apps/
  dashboard/        # Next.js + TypeScript + shadcn/ui  (admin/gestion)
  web-client/       # React + Monaco Editor + xterm.js  (IDE in-browser)
  api/              # Rust — Axum + SQLx + tokio
services/
  orchestrator/     # Rust — cycle de vie containers (bollard)
  worker/           # Rust — Redis Streams consumer (jobs async)
  git-manager/      # Rust — clone/branches éphémères (git2)
  gateway/          # Rust — client sozu-command-lib (gestion ExposureRules)
packages/
  shared-types/     # Types TypeScript partagés (dashboard + web-client)
  api-client/       # Client TypeScript généré depuis OpenAPI
infra/
  docker/           # Dockerfiles + docker-compose.yml
  harness/          # Pipelines Harness (YAML)
  migrations/       # sqlx-migrate — fichiers SQL versionnés
docs/               # Architecture + schémas Mermaid
```

## Commandes utiles
```bash
sudo docker compose up -d          # Lancer l'environnement de dev
sqlx migrate run                   # Appliquer les migrations
cargo test --workspace             # Tests unitaires Rust
cargo build --release              # Build production
sozuctl status                     # État du proxy sozu
```
