# Pull Request

## Résumé

<!-- 1-3 phrases — quoi et pourquoi. Le diff montre le comment. -->

## Issues liées

<!-- Obligatoire — référencer le(s) ID(s) issues.md impacté(s) -->
- Closes KODA-XXX
- Relates to KODA-XXX

**Phase ROADMAP :** `v0.X.0`

## Type de changement

`[ ] Bug fix  [ ] Feature  [ ] Refactoring  [ ] Migration DB  [ ] CI/CD  [ ] Documentation  [ ] Sécurité`

## Services / modules modifiés

<!-- Lister les crates/packages/apps touchés -->
- [ ] `services/api`
- [ ] `services/orchestrator`
- [ ] `services/worker`
- [ ] `services/git-manager`
- [ ] `services/gateway`
- [ ] `services/mcp-gateway`
- [ ] `apps/dashboard`
- [ ] `apps/web-client`
- [ ] `packages/mcp-connectors`
- [ ] `packages/themes`
- [ ] `infra/`

## Migrations DB

- [ ] Aucune migration
- [ ] Migration(s) ajoutée(s) dans `infra/migrations/` — nommage `YYYYMMDDHHMM_<objet>_<action>.sql`
- [ ] `sqlx migrate revert` testé
- [ ] Colonnes NOT NULL ajoutées en 3 temps (nullable → backfill → constraint)

## Checklist sécurité

- [ ] Aucun secret, token ou credential committé
- [ ] Tous les nouveaux endpoints ont une vérification RBAC (org + team)
- [ ] Les nouveaux containers workspace ont les labels `koda.*` obligatoires + resource limits
- [ ] Les nouvelles routes DB filtrent par `organization_id`
- [ ] Les headers `X-Forwarded-*` ne sont trustés que depuis `TRUSTED_PROXY_CIDRS`
- [ ] Les configs MCP/secrets passent par `SecretRef` (jamais en clair)
- [ ] Le volume PersonalSpace reste monté read-only
- [ ] Les fichiers `ai/` du PersonalSpace ne sont jamais loggués

## Checklist code

- [ ] Pas de `SELECT *` sur les tables métier
- [ ] Pas d'appel MCP direct depuis l'API (passer par Redis Streams `jobs:mcp`)
- [ ] Pas de socket Docker brut (passer par docker-socket-proxy)
- [ ] Pas de gestion TLS côté applicatif (sozu s'en charge)
- [ ] Pas de modification sozu par fichier config (passer par `sozu-command-lib`)
- [ ] Types d'erreurs Rust : `anyhow` pour les binaires, `thiserror` pour les libs
- [ ] Config service : `config/default.yaml` + `.env.example` mis à jour si besoin

## Plan de test

<!-- Ce qui a été testé manuellement ou automatiquement -->
- [ ]
- [ ]

## Captures / logs (optionnel)

<!-- Si pertinent pour la review — NE PAS inclure de données sensibles -->
