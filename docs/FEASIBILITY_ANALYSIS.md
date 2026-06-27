# Analyse de Faisabilité — Koda

> Date : 2026-06-27  
> Version specs analysée : SPECIFICATIONS.md (22 étapes)

---

## 1. Résumé exécutif

Koda est un système de gestion d'environnements de développement à la demande, comparable conceptuellement à Gitpod, Coder ou DevPod, mais auto-hébergé sur VPS. L'ambition est réaliste pour un MVP en 6-9 mois avec une équipe de 2-4 développeurs, à condition de prioriser rigoureusement le périmètre et d'adresser plusieurs risques techniques identifiés ci-dessous.

**Verdict global : faisable, avec des zones de risque localisées et surmontables.**

---

## 2. Analyse par couche

### 2.1 Gateway / Reverse-Proxy dynamique

**Faisabilité : ✅ Élevée**

Le routage par UID avec Path Stripping est un pattern standard. Cependant, implémenter un reverse-proxy maison (mentionné comme "reverse-proxy dynamique") représente un effort disproportionné.

**Risque identifié :** Un reverse-proxy custom devra gérer les WebSockets (obligatoire pour code-server / VSCode Browser, JetBrains Gateway, terminaux xterm.js). C'est non-trivial et source de bugs subtils.

**Recommandation :** Utiliser **Traefik v3** comme Gateway plutôt qu'une implémentation custom.
- Routing dynamique via API REST ou labels Docker sans rechargement.
- Path Stripping natif (`StripPrefix` middleware).
- WebSocket transparent sans configuration supplémentaire.
- Tableau de bord intégré (visibilité immédiate des routes actives).
- TLS automatique via Let's Encrypt.

L'API Control Plane met à jour les routes Traefik via son API provider HTTP ou Redis, sans redémarrage.

---

### 2.2 Orchestrateur (cycle de vie des workspaces)

**Faisabilité : ⚠️ Modérée — risque principal du projet**

La gestion des conteneurs Docker depuis un service applicatif est réalisable via le **SDK Docker Python** (`docker-py`). Mais la spec interdit le Docker-in-Docker (DinD), ce qui est la bonne décision. Cela implique que l'orchestrateur accède au socket Docker de l'hôte.

**Risque critique : socket Docker = accès root équivalent.**
Tout processus ayant accès à `/var/run/docker.sock` peut obtenir les droits root sur l'hôte hôte. Si l'API est compromise, toute l'infrastructure l'est.

**Recommandations :**
1. **Court terme MVP :** Isoler l'orchestrateur dans un conteneur dédié avec accès restreint au socket (via proxy `docker-socket-proxy` qui filtre les API autorisées).
2. **Moyen terme :** Évaluer **Podman rootless** ou le runtime **Sysbox** pour une isolation réelle sans socket root.
3. **Définir des limites de ressources obligatoires** sur chaque conteneur workspace (CPU, RAM, PID) via les paramètres Docker — absent des specs actuelles.

**Absence dans les specs :** La gestion du cycle de vie des **volumes Docker** (création, montage, nettoyage) n'est pas formalisée. Proposer une entité `WorkspaceVolume` (voir section 6).

---

### 2.3 Gestionnaire Git (Data & Config)

**Faisabilité : ✅ Élevée**

Le clonage asynchrone avec machine d'états (`pending → cloning → ready | failed`) est un pattern solide. `GitPython` ou `pygit2` couvrent les besoins.

**Points à clarifier :**
- **Taille des dépôts** : un monorepo de 20 Go bloquera un worker Celery longtemps. Prévoir un timeout configurable et un shallow clone optionnel (`--depth 1`).
- **Credentials SSH** : les clés SSH injectées via `SecretRef` doivent être écrites temporairement sur disque dans un répertoire à permissions restreintes, puis supprimées après le clone. Documenter ce flux précisément.
- **Branches éphémères** : la spec mentionne des branches éphémères pour les pipelines CI/CD. Formaliser : création automatique au démarrage du pipeline (`pipeline/<uid>/<timestamp>`), suppression après merge/rejet.

---

### 2.4 API Control Plane (FastAPI)

**Faisabilité : ✅ Élevée**

FastAPI + Pydantic + SQLAlchemy est une stack mature et bien adaptée. Les choix sont cohérents.

**Points de vigilance :**

1. **Versioning API :** `/api/v1/` est prévu, bien. Prévoir dès le départ les conventions de dépréciation (`Sunset` header HTTP).

2. **Pagination manquante** dans les endpoints listés. `GET /api/v1/workspaces` sans cursor ou offset deviendra problématique à 35 000 workspaces/mois (horizon 2 ans).

3. **Webhooks entrants** (pour `on_push` AutomationTrigger) : la spec mentionne la signature obligatoire, mais pas la queue de retry en cas d'échec de traitement. Prévoir un stockage temporaire des events webhook avec TTL.

4. **Event streaming :** Les clients dashboard ont besoin de mises à jour en temps réel (statut workspace, logs pipeline). La spec ne définit pas ce canal. Recommandation : **Server-Sent Events (SSE)** sur `/api/v1/workspaces/:uid/events` — plus simple que WebSocket, natif HTTP/2, sans état serveur.

---

### 2.5 Dashboard (Next.js)

**Faisabilité : ✅ Élevée**

Next.js + shadcn/ui + Tailwind est une combinaison solide et accessible. La cible WCAG 2.1 AA est atteignable avec shadcn/ui (composants Radix sous-jacents, accessibles par défaut).

**Risque UX identifié :** Le flux en 8 étapes séquentielles peut être perçu comme lourd pour une première création. Recommander un **mode "quick-start"** où étapes 4 et 5 sont post-pontées, avec un bandeau de progression non bloquant.

---

### 2.6 Task Runner (Celery + Redis)

**Faisabilité : ✅ Élevée**

Celery + Redis est la combinaison standard Python pour les tâches asynchrones. Bien adapté aux pipelines CI/CD.

**Point d'attention :** À 500 000 jobs/mois (horizon 2 ans), Redis comme broker peut montrer ses limites de persistence. Envisager la migration vers **Redis Streams** (persistence native) ou **RabbitMQ** si les garanties "at-least-once" deviennent critiques. Ce n'est pas urgent pour le MVP.

---

### 2.7 Authentification

**Faisabilité : ✅ Élevée avec nuance**

Le schéma est complet (session cookie, OAuth, OTP). Deux points à clarifier :

1. **OTP email vs TOTP :** "OTP email" a une latence de livraison variable (2-30s). Pour le mode MFA, préférer **TOTP via application** (Google Authenticator, Authy) avec l'email comme fallback. Les deux coexistent proprement avec une colonne `totp_secret` nullable.

2. **Rotation des tokens M2M :** La spec mentionne "token court avec rotation" mais sans détailler le mécanisme. Recommander **RFC 7009 (token revocation)** + **refresh token stocké en DB hashé**.

---

## 3. Risques transversaux

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R1 | Socket Docker accessible = vecteur d'escalade | Élevée (si non mitigé) | Critique | `docker-socket-proxy` en MVP, Sysbox à terme |
| R2 | WebSocket non supporté par la gateway custom | Élevée | Élevé | Adopter Traefik v3 |
| R3 | Clonage Git bloquant le worker | Modérée | Modéré | Timeout + shallow clone + worker dédié |
| R4 | Absence de limites ressources container | Élevée | Élevé | Imposer CPU/RAM/PID limits dès le MVP |
| R5 | Dépendance LLM sans abstraction | Modérée | Modéré | `AiProviderAdapter` mentionné dans spec — à implémenter dès le début |
| R6 | Absence d'événements temps réel côté client | Élevée | Modéré | SSE sur l'API, sinon polling agressif |
| R7 | Volumes Docker orphelins | Élevée (long terme) | Modéré | Garbage collector planifié (Celery beat) |
| R8 | Multi-tenant data leak par oubli de filtre | Modérée | Critique | Row Level Security PostgreSQL en complément des filtres applicatifs |

---

## 4. Incohérences et lacunes dans les specs

### 4.1 Entité manquante : `WorkspaceVolume`

Les volumes Docker (données persistantes du workspace) ne sont pas modélisés. Un workspace peut être détruit et recréé, mais ses données doivent survivre. Sans entité dédiée, le nettoyage est impossible à piloter proprement.

**Proposition :**
```
WorkspaceVolume {
  id, workspace_id, volume_name (Docker), size_mb, created_at,
  last_mounted_at, status: attached | detached | archived
}
```

### 4.2 Manque de définition : ressources container

Aucune spec de limites de ressources par workspace ou par organisation. Sans ça, un workspace peut consommer tout le CPU de l'hôte.

**Proposition :** Ajouter à `Template` ou `WorkspacePluginBinding` :
```
cpu_limit (millicores), memory_limit_mb, pid_limit, storage_limit_gb
```

### 4.3 Ambiguïté : `PluginDefinition` vs `Template`

Les specs distinguent Template (image Docker / runtime) et Plugin (outil d'accès), mais le déclenchement du conteneur semble lié au plugin (`WorkspacePluginBinding déclenche le provisioning container`). C'est une confusion : le conteneur doit être lancé avec l'image du Template ET configuré pour le Plugin.

**Clarification :** Le conteneur est instancié depuis l'image du `Template`, le `Plugin` y est installé/activé, les `ExposureRule` sont créées depuis le Plugin.

### 4.4 Webhook entrant sans stockage d'event

Le trigger `on_push` reçoit un webhook Git. Si le système est momentanément surchargé, l'event est perdu. La spec ne prévoit pas de queue d'events entrants.

**Proposition :** Ajouter une table `IncomingWebhookEvent` (TTL 7 jours) pour stockage avant traitement par le worker.

### 4.5 Absence de stratégie de health check

Rien ne définit comment la plateforme détecte qu'un workspace est réellement prêt (vs. juste "conteneur démarré"). Le conteneur peut démarrer mais le service interne (code-server, JetBrains) peut mettre 15-30s de plus.

**Proposition :** Mécanisme de health probe configurable par plugin (`GET /healthz` sur le port interne, timeout configurable), avec polling du worker jusqu'à succès ou timeout global.

---

## 5. Améliorations de l'architecture proposées

### 5.1 Row Level Security PostgreSQL

En complément des filtres `WHERE organization_id = ?` applicatifs, activer le RLS PostgreSQL sur les tables critiques (`workspaces`, `cicd_pipelines`, `tickets`, `audit_events`). Double filet de sécurité contre les bugs de filtre applicatif.

### 5.2 OpenTelemetry dès le départ

Instrumenter chaque service (FastAPI, Celery workers, Gateway) avec **OpenTelemetry** dès le MVP. Le coût est faible à l'ajout initial, exorbitant en retrofit. Exporter vers Jaeger (self-hosted) ou OTLP compatible.

### 5.3 Architecture événementielle interne

Utiliser **Redis Pub/Sub** (ou un bus d'events interne) pour la communication entre l'orchestrateur et l'API, plutôt que du polling DB. L'API émet un event `workspace.started`, le worker Celery l'écoute et met à jour la gateway.

### 5.4 SSE pour les mises à jour temps réel

Endpoint `GET /api/v1/workspaces/:uid/events` en Server-Sent Events. Le dashboard s'abonne et reçoit les transitions de statut, les logs de pipeline, les alertes — sans polling ni WebSocket complexe.

### 5.5 Workspace `devcontainer.json` natif

Supporter le standard **Dev Container Spec** (`.devcontainer/devcontainer.json`) comme source de configuration du Template/Plugin. Permet la compatibilité avec VS Code Dev Containers et des milliers de repos existants déjà configurés.

### 5.6 `docker-socket-proxy` obligatoire en MVP

Le service orchestrateur ne doit JAMAIS avoir accès au socket Docker brut. Intercaler **Tecnativa/docker-socket-proxy** qui filtre les appels API Docker à la liste blanche (create, start, stop, exec, inspect containers — pas d'accès aux images système ni aux réseaux hôte).

### 5.7 Alembic : conventions de migration renforcées

Ajouter à la convention existante :
- Chaque migration doit avoir un `downgrade()` non-destructif.
- Les migrations de colonnes NOT NULL doivent passer par expand/contract sur 3 déploiements.
- Une migration ne peut pas DROP une colonne sans un délai de 2 semaines après déprecation applicative.

---

## 6. Nouvelles fonctionnalités proposées

### 6.1 Port Forwarding à la demande (MVP+)

Permettre aux utilisateurs d'exposer des ports supplémentaires depuis l'intérieur du workspace, sans redémarrage. L'ExposureRule est créée dynamiquement via l'API.

Interface : bandeau dans le dashboard "Exposer le port X" → URL publique générée instantanément via `/[UID]/port/[PORT]`.

### 6.2 Webhook Inbox par workspace

Chaque workspace reçoit une URL unique `https://domain.com/[UID]/webhook/[TOKEN]` qui capture les webhooks entrants (GitHub, Stripe, Slack...) et les rend consultables dans le dashboard avec le corps complet. Idéal pour déboguer des intégrations sans ngrok.

### 6.3 Workspace Forking

Créer un nouveau workspace depuis l'état courant d'un workspace existant : clone du volume, même branche Git, mêmes ExposureRules. Cas d'usage : expérimentation sans risque, pair programming isolé.

### 6.4 Environnement Variables UI (Env Manager)

Éditeur visuel des variables d'environnement du workspace avec :
- Champ masqué pour les secrets.
- Diff entre les variables actuelles et celles du Template par défaut.
- Import depuis un fichier `.env` (parse local, jamais envoyé tel quel au serveur).

### 6.5 Terminaux partagés (Backlog → MVP+)

Terminaux multiplexés via **WebSocket + xterm.js**, avec sessions nommées et accès partageable via lien temporaire. Fondamental pour le pair programming sans IDE complet.

### 6.6 Snapshot chaud + Restauration rapide

Mécanisme de checkpoint du conteneur (via `docker pause` + copie du volume) pour des rollbacks en secondes. Différent d'un arrêt propre, utile avant une opération risquée.

### 6.7 Pre-warming d'images

Planifier le pull des images Docker les plus utilisées sur l'hôte avant la demande utilisateur. Réduit le cold start de 30-120s à quelques secondes. Géré par un job Celery Beat quotidien sur la liste des templates populaires.

### 6.8 Pipeline IA : Review automatique de diff

Avant la phase de revue (étape 7), déclencher automatiquement un job IA qui analyse le diff Git et produit :
- Résumé des changements en langage naturel.
- Risques potentiels identifiés (sécurité, perf, breaking changes).
- Suggestions de nommage / refactoring.

Affiché en sidebar dans la vue Diff du dashboard.

### 6.9 Workspace Activity Feed

Timeline par workspace de toutes les actions : clonage Git, démarrages/arrêts, exécutions pipeline, commits poussés, tickets créés. Permet à un manager ou reviewer de comprendre l'historique sans interroger les logs bruts.

### 6.10 API de Quotas par Organisation

Permettre aux administrateurs de définir des quotas :
- Nombre maximum de workspaces actifs simultanément.
- Durée maximale d'une session sans activité avant hibernation.
- Limite de CPU/RAM cumulée pour l'organisation.

Géré par une entité `OrganizationQuota` et appliqué à la création de workspace.

### 6.11 CLI Koda (Accès SSH natif)

Un client CLI (`koda connect <uid>`) qui établit un tunnel SSH vers le workspace sans passer par l'interface web. Pour les développeurs préférant leur terminal local avec leurs propres outils.

---

## 7. Roadmap de faisabilité recommandée

### Phase 0 — Fondations (semaines 1-4)
- Monorepo initialisé (apps/, services/, packages/, infra/).
- PostgreSQL + Alembic + modèles de base.
- FastAPI skeleton + authentification (session + OAuth).
- Docker Compose de développement.

### Phase 1 — Workspace minimal (semaines 5-10)
- Création workspace + UID.
- Clone Git asynchrone (Celery).
- Lancement conteneur via docker-socket-proxy.
- Traefik dynamique : route `[UID]/[path]` → container.
- Dashboard : liste workspaces + statut.

### Phase 2 — Workspace complet (semaines 11-16)
- Plugin binding + health probe.
- ExposureRules dynamiques.
- Diff viewer.
- SSE pour statuts temps réel.

### Phase 3 — Pipelines CI/CD (semaines 17-22)
- CiCdPipeline + AutomationTrigger.
- Worker Celery pour exécution pipeline dans conteneur isolé.
- Webhook entrant signé.

### Phase 4 — Sécurité & Observabilité (semaines 23-26)
- RBAC complet + audit events.
- OpenTelemetry + Sentry.
- Tests E2E Playwright (parcours critiques).
- Review sécurité (OWASP Top 10 checklist).

---

## 8. Checklist de cohérence finale

| Critère | Statut | Note |
|---------|--------|------|
| Stack compatible self-hosted VPS | ✅ | Docker Compose + Traefik + PostgreSQL |
| Contrat API aligné entités métier | ✅ avec gaps | Pagination et SSE à ajouter |
| Sécurité multi-tenant | ⚠️ | RLS PostgreSQL recommandé en plus des filtres applicatifs |
| Migrations rollback-safe | ✅ | Alembic + expand/contract |
| WCAG 2.1 AA dashboard | ✅ | shadcn/ui (Radix) accessible par défaut |
| Budget hébergement plausible | ✅ | 75-180 EUR/mois réaliste pour MVP |
| Isolation container | ⚠️ | docker-socket-proxy obligatoire + resource limits |
| WebSocket gateway | ⚠️ | Nécessite Traefik ou configuration proxy explicite |
| Volumes persistants formalisés | ❌ | Entité `WorkspaceVolume` à ajouter aux specs |
| Health checks workspace | ❌ | Mécanisme de probe à définir par plugin |
