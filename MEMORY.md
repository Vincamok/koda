# MEMORY.md — Koda

## Contexte projet
Koda est une plateforme de gestion d'environnements de développement à la demande, auto-hébergée sur VPS. Chaque workspace est un conteneur Docker isolé, accessible via une URL unique `domain.com/[UID]/[service]` gérée par nginxify (reverse-proxy nginx dynamique avec API).

## Décisions architecturales actées

| Décision | Choix retenu | Alternative écartée |
|----------|-------------|---------------------|
| Gateway | **nginxify** (nginx + API, outil existant) | Traefik v3, proxy custom |
| Port forwarding | Géré nativement par nginxify | Implémentation custom |
| Backend API | **Axum** (Rust) + SQLx + tokio | FastAPI (Python) |
| Workers / Task Runner | **Rust** (tokio + Redis Streams) | Celery (Python) |
| Docker SDK (Rust) | **bollard** (Docker API async) | docker-py |
| Git (Rust) | **git2** (libgit2 bindings) | shell git |
| LLM integration | **reqwest** + HTTP direct (AiProviderAdapter trait) | SDK Python |
| Frontend | **Next.js** + TypeScript + shadcn/ui + Tailwind | SvelteKit |
| BDD | PostgreSQL | — |
| Migrations | **sqlx-migrate** (fichiers SQL versionnés) | Alembic |
| Isolation socket Docker | docker-socket-proxy (whitelist API) | Socket brut |
| Queue broker | Redis Streams (consumer groups) | RabbitMQ |

## Entités métier clés
- `Workspace` : instance identifiée par UID immuable. Statuts : `created → configuring → running → reviewing → closing → closed`
- `WorkspaceGitConfig` : config Git du workspace (1 actif max), clone_status : `pending → cloning → ready | failed`
- `WorkspacePluginBinding` : plugin actif déclenchant le provisioning container. Statuts : `installing → ready | failed`
- `ExposureRule` : mapping publicPath ↔ internalPort (stripPrefix=true). Créé par le plugin, appliqué via nginxify API
- `WorkspaceVolume` : volume Docker persistant lié au workspace *(ajout vs specs initiales)*
- `CiCdPipeline` : pipeline build/lint/security. Statuts : `idle → running → passed | failed`
- `AutomationTrigger` : déclencheur on_push | schedule | manual
- `IncomingWebhookEvent` : event webhook entrant stocké (TTL 7j) avant traitement worker *(ajout vs specs initiales)*
- `OrganizationQuota` : limites de ressources par organisation *(ajout vs specs initiales)*
- `AuditEvent` : traçabilité de toutes les actions critiques

## Contraintes non-négociables
- Pas de Docker-in-Docker (DinD)
- Path Stripping obligatoire via nginxify (l'app ne voit jamais le préfixe /[UID]/)
- organization_id obligatoire sur toutes les entités exposées (+ RLS PostgreSQL)
- Aucun secret stocké en clair (SecretRef uniquement)
- Limites CPU/RAM/PID obligatoires sur chaque conteneur workspace (bollard CreateContainerOptions)

## Risques principaux
1. Socket Docker = vecteur root → mitigé par docker-socket-proxy
2. WebSocket (code-server, xterm.js) → géré par nginxify (nginx proxy_pass + upgrade)
3. Absence de health probe par plugin → mécanisme probe défini par plugin (GET /healthz)
4. Volumes orphelins → garbage collector planifié (worker Rust Cron)
5. LLM sans abstraction → AiProviderAdapter trait dès le départ

## Stack de développement
```
apps/
  dashboard/        # Next.js + TypeScript + shadcn/ui
  api/              # Rust — Axum + SQLx + tokio
services/
  orchestrator/     # Rust — cycle de vie containers (bollard)
  worker/           # Rust — Redis Streams consumer (jobs async)
  git-manager/      # Rust — clone/branches éphémères (git2)
packages/
  shared-types/     # Types TypeScript partagés (dashboard)
  api-client/       # Client TypeScript généré depuis OpenAPI
infra/
  docker/           # Dockerfiles + docker-compose.yml
  ci/               # Pipelines CI/CD
  migrations/       # sqlx-migrate — fichiers SQL versionnés
docs/               # Architecture + schémas Mermaid
```

## Commandes utiles
```bash
sudo docker compose up -d          # Lancer l'environnement de dev
sqlx migrate run                   # Appliquer les migrations
cargo test --workspace             # Tests unitaires Rust
cargo build --release              # Build production
```
