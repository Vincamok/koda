---
name: "Task / Chore"
about: "Tâche technique, refactoring, dette technique ou infrastructure"
title: "[TASK] "
labels: ["task", "triage"]
assignees: []
---

## Type de tâche

`[ ] Refactoring  [ ] Migration DB  [ ] Infrastructure  [ ] CI/CD  [ ] Documentation  [ ] Tests  [ ] Dette technique  [ ] Dépendances`

## Contexte

**Phase cible selon ROADMAP.md :**
<!-- v0.1.0 | v0.2.0 | v0.3.0 | v0.4.0 | v1.0.0 -->

**ID issue liée dans issues.md :**
<!-- Obligatoire — chaque tâche doit correspondre à un item dans issues.md. Ex: KODA-015, KODA-D02 -->

**Service / module concerné :**

## Description

<!-- Quoi faire et pourquoi — se concentrer sur le WHY (la contrainte, l'invariant, le bug silencieux) -->

## Définition de "Done"

- [ ]
- [ ]
- [ ]

## Migrations DB (si applicable)

- **Fichiers :** `infra/migrations/YYYYMMDDHHMM_<objet>_<action>.sql`
- [ ] `sqlx migrate revert` testé en staging
- [ ] Colonne NOT NULL : nullable → backfill → constraint (3 temps)
- [ ] Pas de DROP sans 2 semaines post-déprecation

## Tests à ajouter / modifier

<!-- Préciser le module de test et le cas couvert -->

## Risques / points d'attention

<!-- Régressions potentielles, dépendances inter-services, ordre d'exécution requis -->

## Checklist avant soumission

- [ ] L'ID `issues.md` est renseigné et valide
- [ ] Les règles invariantes de `AGENTS.md` sont respectées
- [ ] Aucun secret ou credential dans la description
