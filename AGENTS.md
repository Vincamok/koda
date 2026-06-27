# AGENTS.md — Koda

## Règles pour les agents IA travaillant sur ce projet

### Contexte général
Koda est une plateforme de workspaces de développement à la demande. Chaque workspace est un conteneur Docker isolé, accessible via `domain.com/[UID]/[service]`. Lire `MEMORY.md` et `docs/FEASIBILITY_ANALYSIS.md` avant toute modification.

### Règles invariantes

1. **UID immuable** : ne jamais modifier l'UID d'un workspace après création. C'est la clé de routage réseau.
2. **organization_id** : toute requête DB sur une entité métier doit filtrer par `organization_id`. Pas d'exception.
3. **Secrets** : ne jamais logger, sérialiser ou stocker un secret en clair. Toujours passer par `SecretRef`.
4. **Path Stripping** : les applications dans les containers ne doivent jamais recevoir le préfixe `/[UID]/`. La gateway strip ce préfixe.
5. **Docker socket** : l'orchestrateur ne passe que par `docker-socket-proxy`. Ne jamais exposer le socket brut.
6. **Resource limits** : tout conteneur workspace lancé doit avoir `cpu_limit`, `memory_limit`, `pids_limit` définis.

### Conventions de code

**Backend (FastAPI)**
- Toutes les routes sous `/api/v1/`
- Réponse succès : `{"data": ..., "meta": ...}`
- Réponse erreur : `{"error": {"code": "SNAKE_CASE_CODE", "message": "...", "requestId": "..."}}`
- Validation Pydantic sur tous les inputs
- Dépendances injectées via `Depends()`

**Frontend (Next.js)**
- Composants dans `apps/dashboard/src/components/`
- Appels API via un client centralisé (jamais de `fetch` brut dans les composants)
- Internationalisation via clés i18n dès le départ (même si mono-langue FR au MVP)
- Accessibilité WCAG 2.1 AA : labels explicites, focus visible, contraste suffisant

**Migrations Alembic**
- Nommage : `YYYYMMDDHHMM_<objet>_<action>` (ex: `202604302245_workspace_status_index`)
- Chaque migration doit avoir un `downgrade()` non-destructif
- DROP de colonne interdit sans délai de 2 semaines post-déprecation applicative

**Git**
- Branches : `feature/*`, `fix/*`
- Branches de pipeline CI éphémères : `pipeline/<uid>/<timestamp>`

### Entités — ordre de dépendance pour les migrations
```
Organization → User → Membership
Organization → Project → Template → Workspace
Workspace → WorkspaceGitConfig
Workspace → WorkspaceVolume
Workspace → WorkspacePluginBinding → ExposureRule
Workspace → CiCdPipeline → AutomationTrigger
Workspace → TicketRecord
Workspace → SecretRef
User → AuditEvent
```

### Ce qu'il ne faut PAS faire
- Ajouter du Docker-in-Docker
- Committer des fichiers `.env` ou contenant des secrets
- Créer des endpoints sans vérification RBAC
- Lancer un conteneur workspace sans limits de ressources
- Accéder au socket Docker autrement que via docker-socket-proxy
- Faire un `SELECT *` sur des tables métier sans filtre `organization_id`
