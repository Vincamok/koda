# MEMORY.md — Koda

## Contexte projet
Koda est une plateforme de gestion d'environnements de développement à la demande, auto-hébergée sur VPS. Chaque workspace est un conteneur Docker isolé, accessible via une URL unique `domain.com/[UID]/[service]` gérée par un reverse-proxy dynamique.

## Décisions architecturales actées

| Décision | Choix retenu | Alternative écartée |
|----------|-------------|---------------------|
| Gateway | Traefik v3 (dynamique, WebSocket natif) | Proxy custom |
| Backend API | FastAPI + Pydantic + SQLAlchemy 2.0 | Django DRF |
| Queue | Celery + Redis | BullMQ |
| Frontend | Next.js + TypeScript + shadcn/ui + Tailwind | — |
| BDD | PostgreSQL | — |
| Migrations | Alembic | — |
| Isolation socket Docker | docker-socket-proxy (whitelist API) | Socket brut |

## Entités métier clés
- `Workspace` : instance identifiée par UID immuable. Statuts : `created → configuring → running → reviewing → closing → closed`
- `WorkspaceGitConfig` : config Git du workspace (1 actif max)
- `WorkspacePluginBinding` : plugin actif déclenchant le provisioning container
- `ExposureRule` : mapping publicPath ↔ internalPort (stripPrefix=true par défaut)
- `WorkspaceVolume` : volume Docker persistant lié au workspace *(à ajouter aux specs)*
- `CiCdPipeline` : pipeline build/lint/security. Statuts : `idle → running → passed | failed`
- `AutomationTrigger` : déclencheur on_push | schedule | manual
- `AuditEvent` : traçabilité de toutes les actions critiques

## Contraintes non-négociables
- Pas de Docker-in-Docker (DinD)
- Path Stripping obligatoire au niveau gateway (l'app ne voit jamais le préfixe /[UID]/)
- organization_id obligatoire sur toutes les entités exposées
- Aucun secret stocké en clair
- Limites CPU/RAM/PID obligatoires sur chaque conteneur workspace

## Risques principaux
1. Socket Docker = vecteur root → mitigé par docker-socket-proxy
2. WebSocket non supporté sans configuration proxy explicite → Traefik gère nativement
3. Absence de health probe par plugin → mécanisme à définir
4. Volumes orphelins → garbage collector Celery Beat planifié

## Stack de développement
```
apps/
  dashboard/     # Next.js
  api/           # FastAPI
services/
  orchestrator/  # Gestion cycle de vie containers
  worker/        # Celery workers
  gateway/       # Config Traefik dynamique
packages/
  shared-types/  # Types TypeScript partagés
  config/        # Config centralisée
infra/
  docker/        # Dockerfiles + compose
  ci/            # Pipelines CI/CD
docs/            # Architecture + schémas
```

## Commandes utiles
```bash
sudo docker compose up -d    # Lancer l'environnement de dev
```
