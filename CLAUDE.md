# CLAUDE.md — Instructions permanentes pour Claude Code

## Règle fondamentale — Branches

**Tout mettre dans `main`. Pas de branches. Jamais.**
- Tous les commits vont directement sur `main`
- `git push -u origin main` après chaque commit
- Ne jamais créer de branche de travail

---

## Règle fondamentale — Versioning et changelog

**À chaque modification de fichiers du projet, appliquer systématiquement :**

### 1. Incrémenter la version (SemVer)

Logique décrite dans ROADMAP.md et CHANGELOG.md :

```
MAJOR.MINOR.PATCH
- 0.x.x : développement pré-MVP
- 1.0.0 : MVP stable (Phase 4 complète)
- Chaque phase = une version mineure (0.1, 0.2, 0.3…)
- PATCH : bug fix, ajout mineur, correctif dans une phase
- MINOR : phase complète franchie
- MAJOR : pas encore (MVP non atteint)
```

Règles de décision :
- Nouveau endpoint / nouveau composant / nouvelle feature → **PATCH** dans la phase courante
- Phase entière complétée → **MINOR** (ex. `0.3.0` → `0.4.0`)
- Bug fix / correctif → **PATCH**

### 2. Mettre à jour CHANGELOG.md

Format Keep a Changelog :

```markdown
## [X.Y.Z] — YYYY-MM-DD · <Titre phase>

### Added
- Description concise de chaque ajout

### Changed
- Ce qui a changé dans un comportement existant

### Fixed
- Bug corrigés

### Removed
- Ce qui a été supprimé
```

- Les entrées en cours vont dans `## [Unreleased]`
- Quand une version est taguée, `[Unreleased]` devient `[X.Y.Z] — date`

### 3. Mettre à jour ROADMAP.md

- Cocher `[x]` les items livrés dans la phase courante
- Ne jamais supprimer les items non livrés
- Mettre à jour le statut de la phase si elle est complète (`✓`)

---

## Règle — Sécurité

- **Aucun secret en clair** — `SecretRef` uniquement
- **Docker socket via proxy** (`docker-socket-proxy`) — jamais le socket brut
- **Resource limits** obligatoires sur tous les containers (cpu_period, cpu_quota, memory, pids_limit)
- **RLS PostgreSQL** actif sur 13+ tables critiques
- **Rate limiting** par IP (300/min) et par utilisateur (600/min)
- **`organization_id`** : toute requête DB sur une entité métier doit filtrer par `organization_id`
- **Secrets fichiers** : `.env`, `*.key`, `*.pem`, clés SSH → jamais transmis au LLM
- **Labels containers** : `koda.managed`, `koda.type`, `koda.workspace_id`, `koda.org_id` obligatoires
- **Ne jamais logger** les valeurs de config reçues par `call_tool` (peut contenir des tokens)
- **`super_admin`** : toute impersonation génère un `AuditEvent`

---

## Règle — Architecture

- **Monorepo** : `apps/` (api, dashboard, web-client, admin), `services/` (orchestrator, worker, git-manager, gateway, mcp-gateway), `packages/` (shared-types, api-client, i18n, mcp-connectors), `infra/`
- **API** : Axum 0.7 + SQLx 0.7 + PostgreSQL
- **Dashboard / Web-client / Admin** : Next.js 14 App Router + next-intl
- **Multi-tenant** : `organization_id` sur toutes les entités métier
- **Migrations** : `infra/migrations/YYYYMMDDHHMMSS_description.sql` — jamais modifier une migration appliquée
- **i18n** : FR/EN/ES/DE sur les 3 apps Next.js — dashboard charge `apps/dashboard/messages/`, web-client charge `packages/i18n/messages/`

---

## Règle — Commit

Format de commit :

```
<type>: <description courte>

<détail des changements si nécessaire>

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01QJQeYLcuAvfKsY6bbQroiU
```

Types : `feat`, `fix`, `refactor`, `docs`, `chore`, `test`
