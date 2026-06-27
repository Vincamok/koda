---
name: "Feature Request"
about: "Proposer une nouvelle fonctionnalité ou amélioration"
title: "[FEATURE] "
labels: ["enhancement", "triage"]
assignees: []
---

## Contexte

**Phase cible selon ROADMAP.md :**
<!-- v0.1.0 | v0.2.0 | v0.3.0 | v0.4.0 | v1.0.0 | backlog -->

**ID issue liée dans issues.md :**
<!-- ex: KODA-B07 pour une entrée backlog, ou nouvel item — consulter issues.md avant de créer -->

**Service / module concerné :**
<!-- ex: services/api, services/orchestrator, apps/dashboard, apps/web-client, services/mcp-gateway -->

## Problème à résoudre

<!-- Décrire le besoin utilisateur ou le manque fonctionnel. "En tant que [rôle], je veux [action] afin de [bénéfice]." -->

## Solution proposée

<!-- Description de la feature, comportement attendu, flux utilisateur -->

## Alternatives envisagées

<!-- Autres approches considérées et raison du rejet éventuel -->

## Contraintes architecturales à respecter

<!-- Cocher les règles AGENTS.md impactées -->
- [ ] UID immuable workspace
- [ ] organization_id obligatoire sur toutes les requêtes DB
- [ ] Secrets via SecretRef uniquement
- [ ] Path stripping sozu
- [ ] Resource limits containers Docker
- [ ] Labels koda.* obligatoires
- [ ] RBAC (org + team + WorkspaceShare)
- [ ] Proxy trust TRUSTED_PROXY_CIDRS

## Entités DB impactées

<!-- Lister les entités créées ou modifiées — se référer à l'arbre de dépendances dans AGENTS.md -->

## Migrations nécessaires

<!-- Nommage : YYYYMMDDHHMM_<objet>_<action>.sql -->
- [ ] Aucune
- [ ] Nouvelle(s) table(s) :
- [ ] Colonne(s) ajoutée(s) :
- [ ] Index :

## Checklist avant soumission

- [ ] J'ai consulté `issues.md` pour éviter les doublons
- [ ] J'ai vérifié la cohérence avec `docs/IMPLEMENTATION_SPEC.md`
- [ ] La feature respecte les règles invariantes de `AGENTS.md`
- [ ] La feature est compatible avec la phase cible de `ROADMAP.md`
