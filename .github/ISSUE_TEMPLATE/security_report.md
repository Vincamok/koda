---
name: "Security Report"
about: "Signaler une vulnérabilité de sécurité (usage interne uniquement)"
title: "[SECURITY] "
labels: ["security", "critical", "triage"]
assignees: []
---

> **IMPORTANT** : Ne jamais publier de PoC exploitable, token, credential ou payload d'attaque dans ce ticket.
> Pour les vulnérabilités critiques en production, contacter directement l'équipe sécurité hors GitHub.

## Classification

**Type de vulnérabilité :**
`[ ] Injection  [ ] Authent/Session  [ ] RBAC/Autorisation  [ ] Fuite de données  [ ] XSS  [ ] SSRF  [ ] Dépendance  [ ] Config  [ ] Autre`

**Sévérité estimée (CVSS) :**
`[ ] Critique (9.0-10)  [ ] Haute (7.0-8.9)  [ ] Moyenne (4.0-6.9)  [ ] Basse (0.1-3.9)`

**ID issue liée dans issues.md :**
<!-- ex: KODA-D01 pour les dettes connues — consulter issues.md section "Dettes techniques / Bugs connus" -->

## Service / composant

<!-- ex: services/api src/handlers/auth.rs, services/orchestrator, apps/dashboard -->

## Description

<!-- Description générale de la vulnérabilité sans PoC exploitable -->

## Impact potentiel

<!-- Quelles données, quels utilisateurs, quelles orgs sont exposés ? -->

## Conditions d'exploitation

<!-- Quel niveau d'accès est requis ? Authentifié ? Admin ? Réseau interne ? -->

## Suggestion de remédiation

<!-- Piste de correction si connue -->

## Références

<!-- CVE, OWASP, CWE si applicable -->

## Checklist

- [ ] Aucun secret, token ou credential inclus dans ce ticket
- [ ] Aucun PoC permettant l'exploitation directe
- [ ] J'ai vérifié si la vulnérabilité est déjà listée dans `issues.md` (section KODA-D)
- [ ] L'équipe sécurité a été notifiée hors-band si la sévérité est Critique
