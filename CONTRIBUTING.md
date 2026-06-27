# Contributing — Koda

## Avant de commencer

Lire impérativement dans cet ordre :

1. **`MEMORY.md`** — décisions d'architecture prises, rationale, historique
2. **`AGENTS.md`** — règles invariantes pour tous les contributeurs (humains et IA)
3. **`docs/IMPLEMENTATION_SPEC.md`** — spécification détaillée par module
4. **`issues.md`** — tracker des tâches ouvertes, backlog, dettes connues
5. **`ROADMAP.md`** — phases et périmètre de chaque version
6. **`CHANGELOG.md`** — historique des décisions et ajouts

---

## Issues et tickets

### Source de vérité : `issues.md`

`issues.md` est le registre central de toutes les tâches du projet. Avant d'ouvrir un ticket GitHub, vérifier que l'item n'est pas déjà listé :

- **KODA-001 à KODA-101** : issues numérotées par phase
- **KODA-B01 à KODA-B13** : backlog (futur, non planifié)
- **KODA-D01 à KODA-D04** : dettes techniques et bugs connus

Chaque ticket GitHub doit référencer un ID `issues.md`. Si la tâche n'existe pas encore dans `issues.md`, l'y ajouter en premier.

### Templates disponibles

| Template | Usage |
|---|---|
| `bug_report.md` | Comportement inattendu ou régression |
| `feature_request.md` | Nouvelle fonctionnalité ou amélioration |
| `security_report.md` | Vulnérabilité de sécurité (usage interne) |
| `task.md` | Tâche technique, refactoring, migration, CI/CD |

---

## Workflow de développement

### Branches

```
main          — production, déploiement automatique Harness
develop       — intégration, base des feature branches
feat/<id>-<slug>   — ex: feat/koda-015-workspace-volumes
fix/<id>-<slug>    — ex: fix/koda-d02-mcp-redis-publish
chore/<id>-<slug>  — ex: chore/koda-023-migration-scan-rules
```

L'ID `issues.md` dans le nom de branche est obligatoire.

### Commits

Format : `<type>(<scope>): <description courte>`

Types : `feat`, `fix`, `chore`, `refactor`, `test`, `docs`, `ci`, `security`

Scope : nom du service ou module (`api`, `orchestrator`, `worker`, `dashboard`, `web-client`, `mcp-gateway`, `gateway`, `git-manager`, `infra`, `themes`, `mcp-connectors`)

Exemples :
```
feat(orchestrator): add multi-network creation per workspace
fix(api): filter workspaces by organization_id on list endpoint
chore(infra): add migration 202604302245_workspace_add_status_index
security(api): validate X-Forwarded-For against TRUSTED_PROXY_CIDRS
```

**Jamais dans un commit :**
- Fichiers `.env` ou contenant des secrets
- Credentials, tokens, clés API
- Fichiers générés (`target/`, `node_modules/`, `dist/`)

---

## Pull Requests

Utiliser le template `.github/PULL_REQUEST_TEMPLATE.md`.

**Règles obligatoires :**

- Toute PR doit référencer au moins un ID `issues.md`
- La checklist sécurité doit être complétée (pas juste cochée sans vérification)
- Les migrations DB doivent être testées avec `sqlx migrate revert` en staging avant merge
- Pas de `force push` sur `main` ou `develop`
- Merge via squash ou merge commit selon la taille de la PR (pas de rebase public)

---

## Règles invariantes (résumé)

Se référer à `AGENTS.md` pour la liste complète. Points critiques :

| Règle | Raison |
|---|---|
| UID workspace immuable après création | Clé de routage sozu — toute modification casse les routes |
| `organization_id` sur toutes les requêtes DB métier | Isolation multi-tenant — le RLS PostgreSQL est un filet, pas la première ligne |
| Secrets via `SecretRef` uniquement | Jamais de credential en clair en DB, logs ou mémoire au-delà du call |
| sozu via `sozu-command-lib` uniquement | Jamais éditer les fichiers de config sozu directement |
| Resource limits sur tous les containers workspace | Sans `memory` + `pids_limit`, un workspace peut DoS l'hôte |
| Labels `koda.*` sur tous les containers éphémères | Traçabilité et garbage collection automatique |
| Pas de socket Docker brut | Passer par `docker-socket-proxy` pour isoler les permissions |
| Volume PersonalSpace monté read-only | Isolation — un workspace ne doit pas corrompre l'espace personnel |
| Fichiers `ai/` PersonalSpace jamais loggués | Données personnelles hors du périmètre des logs |
| Headers `X-Forwarded-*` trustés uniquement depuis `TRUSTED_PROXY_CIDRS` | Prévenir le spoofing IP |

---

## Migrations DB

Nommage : `YYYYMMDDHHMM_<objet>_<action>.sql`

Fichiers dans `infra/migrations/`. Chaque fichier est un SQL pur, sans transaction explicite (sqlx-migrate gère).

**Règles :**
- Colonne NOT NULL : 3 migrations distinctes (nullable → backfill → NOT NULL constraint)
- DROP de colonne : interdit sans 2 semaines de déprecation applicative post-merge
- Tester `sqlx migrate revert` avant de soumettre la PR
- Ordre des dépendances : voir section "Entités — ordre de dépendance" dans `AGENTS.md`

---

## Configuration par service

Chaque service a :
- `config/default.yaml` — valeurs par défaut, commité
- `.env.example` — variables d'environnement documentées, commité
- `.env` — valeurs locales, **gitignored**

Merge via `figment` : `env > .env > config/default.yaml`

`APP_BASE_URL` est obligatoire pour tout service générant des URLs absolues.

---

## Tests

- Backend Rust : `cargo test` dans le crate concerné
- Frontend : `pnpm test` ou `pnpm test:e2e` selon le package
- Migrations : `sqlx migrate run` puis `sqlx migrate revert` en staging
- Couverture cible : ≥ 75% global, ≥ 90% pour le code sécurité et routage (objectif v1.0.0)

---

## Sécurité — signalement

Pour les vulnérabilités de sévérité **Critique** ou **Haute** :
- Ne pas ouvrir un ticket public GitHub
- Contacter directement l'équipe via le canal sécurité interne
- Utiliser le template `security_report.md` uniquement pour les findings Moyenne/Basse ou post-correction

---

## Questions

Consulter dans l'ordre : `MEMORY.md` → `AGENTS.md` → `docs/IMPLEMENTATION_SPEC.md` → `issues.md`.

Si la réponse n'y est pas, ouvrir une discussion GitHub ou un ticket `task` avec le label `question`.
