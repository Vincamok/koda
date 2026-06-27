# AGENTS.md — Koda

## Règles pour les agents IA travaillant sur ce projet

### Contexte général
Koda est une plateforme de workspaces de développement à la demande. Chaque workspace est un conteneur Docker isolé, accessible via `domain.com/[UID]/[service]`. Le routage et le path stripping sont gérés par **nginxify** (outil nginx existant avec API). Lire `MEMORY.md` et `docs/FEASIBILITY_ANALYSIS.md` avant toute modification.

### Règles invariantes

1. **UID immuable** : ne jamais modifier l'UID d'un workspace après création. C'est la clé de routage nginxify.
2. **organization_id** : toute requête DB sur une entité métier doit filtrer par `organization_id`. Pas d'exception. Le RLS PostgreSQL est activé en complément.
3. **Secrets** : ne jamais logger, sérialiser ou stocker un secret en clair. Toujours passer par `SecretRef`.
4. **Path Stripping** : les applications dans les containers ne reçoivent jamais le préfixe `/[UID]/`. nginxify strip ce préfixe avant transmission.
5. **Docker socket** : l'orchestrateur ne passe que par `docker-socket-proxy`. Ne jamais utiliser le socket brut.
6. **Resource limits** : tout conteneur workspace lancé via `bollard` doit avoir `cpu_period`, `cpu_quota`, `memory`, `pids_limit` définis dans `HostConfig`.

### Conventions de code

**Backend API (Rust / Axum)**
- Toutes les routes sous `/api/v1/`
- Réponse succès : `{"data": ..., "meta": ...}`
- Réponse erreur : `{"error": {"code": "SNAKE_CASE_CODE", "message": "...", "request_id": "..."}}`
- Validation via types Rust + `validator` crate sur les structs d'input
- Middleware d'authentification sur toutes les routes sauf `/api/v1/auth/*`
- Pagination cursor-based obligatoire sur tous les endpoints de liste
- SSE sur `GET /api/v1/workspaces/:uid/events` pour le temps réel

**Workers Rust (Redis Streams)**
- Consumer groups Redis pour garantie "at-least-once"
- Timeout par job configurable via `config/platform.config.yaml`
- Chaque job loggue au format JSON structuré (OpenTelemetry compatible)
- Un job qui échoue 3 fois est déplacé dans le stream `jobs:dead_letter`

**Migrations (sqlx-migrate)**
- Nommage : `YYYYMMDDHHMM_<objet>_<action>.sql` (ex: `202604302245_workspace_add_status_index.sql`)
- Chaque migration est un fichier SQL pur dans `infra/migrations/`
- Colonne NOT NULL ajoutée en 3 temps : nullable + backfill + NOT NULL constraint
- DROP de colonne interdit sans délai de 2 semaines post-déprecation applicative
- Tester `sqlx migrate revert` en staging avant merge

**Gateway (nginxify)**
- Créer/supprimer une `ExposureRule` = appel à l'API nginxify (jamais éditer nginx.conf directement)
- Format attendu par nginxify : `{ uid, public_path, internal_host, internal_port, strip_prefix: true }`
- Port forwarding ad-hoc géré par nginxify (pas d'implémentation custom)

**Git Manager (Rust / git2)**
- Clonage toujours asynchrone, statut mis à jour en DB à chaque transition
- Clés SSH temporaires : écrites dans `/run/secrets/<workspace_id>/` (tmpfs), supprimées après clone
- Branches pipeline éphémères : `pipeline/<uid>/<timestamp>`, supprimées après merge/rejet

**Frontend (Next.js)**
- Composants dans `apps/dashboard/src/components/`
- Appels API via `apps/dashboard/src/lib/api-client.ts` (jamais de `fetch` brut dans les composants)
- Internationalisation via clés i18n dès le départ (mono-langue FR au MVP)
- Accessibilité WCAG 2.1 AA : labels explicites, focus visible, contraste suffisant

### Entités — ordre de dépendance pour les migrations
```
Organization → User → Membership
Organization → OrganizationQuota
Organization → Project → Template → Workspace
Workspace → WorkspaceGitConfig
Workspace → WorkspaceVolume
Workspace → WorkspacePluginBinding → ExposureRule
Workspace → CiCdPipeline → AutomationTrigger
Workspace → IncomingWebhookEvent
Workspace → TicketRecord
Workspace → SecretRef
User → AuditEvent
```

### Ce qu'il ne faut PAS faire
- Ajouter du Docker-in-Docker
- Committer des fichiers `.env` ou contenant des secrets
- Créer des endpoints sans vérification RBAC
- Lancer un conteneur workspace sans `HostConfig.memory` et `HostConfig.pids_limit`
- Accéder au socket Docker autrement que via docker-socket-proxy
- Faire un `SELECT *` sur des tables métier sans filtre `organization_id`
- Modifier les règles nginxify en éditant nginx.conf directement
- Écrire du code Rust sans types pour les erreurs (`anyhow` pour les binaires, `thiserror` pour les libs)
