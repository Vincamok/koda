# Analyse de Faisabilité — Koda

> Date : 2026-06-27  
> Version specs analysée : SPECIFICATIONS.md (22 étapes)  
> Révision stack : 2026-06-27 (Rust backend + nginxify)

---

## 1. Résumé exécutif

Koda est un système de gestion d'environnements de développement à la demande, comparable conceptuellement à Gitpod, Coder ou DevPod, mais auto-hébergé sur VPS. L'ambition est réaliste pour un MVP en 6-9 mois avec une équipe de 2-4 développeurs.

**Stack retenue :**
- Backend API + Workers : **Rust** (Axum + SQLx + tokio + bollard)
- Gateway : **sozu** (reverse-proxy Rust, TLS + HTTP/2 + TCP natifs)
- Frontend : **Next.js** + TypeScript + shadcn/ui + Tailwind
- BDD : **PostgreSQL** + sqlx-migrate
- Queue : **Redis Streams** (consumer groups Rust)

**Verdict global : faisable, avec des zones de risque localisées et surmontables.**

---

## 2. Analyse par couche

### 2.1 Gateway / Reverse-Proxy dynamique — sozu

**Faisabilité : ✅ Élevée**

**sozu** est un reverse-proxy Rust développé par Clever Cloud, utilisé en production. Il remplace entièrement nginxify et couvre tous les besoins Koda sans dépendance externe non-Rust :

| Besoin | sozu |
|--------|------|
| Routing HTTP par chemin `[UID]/[service]` | ✅ `HttpFrontend` + `path_prefix` |
| Path Stripping | ✅ natif |
| WebSocket (code-server, xterm.js) | ✅ HTTP Upgrade transparent |
| TLS + Let's Encrypt | ✅ intégré |
| HTTP/2 | ✅ intégré |
| Tunnels TCP (SSH, PostgreSQL) | ✅ `TcpFrontend` |
| Rechargement sans coupure | ✅ hot reload natif |
| Configuration dynamique | ✅ `sozu-command-lib` (crate Rust) |

**Intégration Koda → sozu :**

Le service `services/gateway/` utilise `sozu-command-lib` pour piloter sozu programmatiquement. Aucun fichier de config à éditer, aucun reload manuel.

Routes HTTP (multiplexage par chemin) :
```rust
// Ajout d'une ExposureRule HTTP
sozu.add_http_frontend(HttpFrontend {
    path_prefix: format!("/{uid}/app"),
    ..
});
sozu.add_backend(Backend { address: "172.17.0.X:3000", .. });
```

Routes TCP (multiplexage par port) :
```rust
// Tunnel SSH workspace : port 2201 → container:22
sozu.add_tcp_frontend(TcpFrontend { port: 2201, .. });
sozu.add_backend(Backend { address: "172.17.0.X:22", .. });
```

**Allocation de ports TCP :** plages réservées stockées en DB dans `ExposureRule.host_port`.
- SSH : `2200–2999`
- PostgreSQL : `5400–5499`

**Sécurité :** sozu écoute sur un socket Unix local — pas d'API réseau exposée. L'accès au socket est limité au service `gateway/` dans le même Docker network.

---

### 2.2 Orchestrateur (cycle de vie des workspaces)

**Faisabilité : ✅ Élevée en Rust**

Le crate **bollard** fournit un client Docker API async natif Rust (tokio). Il couvre toutes les opérations nécessaires : `create_container`, `start_container`, `stop_container`, `inspect_container`, `remove_container`, `create_volume`, `remove_volume`.

**Avantage Rust ici :** la compilation typée garantit que les `HostConfig` (limites ressources) sont correctement formés avant envoi à Docker. Un oubli de `memory` ou `pids_limit` est une erreur de compilation si les champs sont `NonZero`.

**Risque critique inchangé : socket Docker = accès root équivalent.**

**Recommandations :**
1. **MVP :** Intercaler `docker-socket-proxy` (Tecnativa) entre bollard et le daemon. Whitelist : `containers/`, `volumes/`, `images/` (lecture seule). Aucun accès aux `networks` hôte ni aux `swarm`.
2. **Moyen terme :** Évaluer Podman rootless ou Sysbox pour isolation sans socket root.
3. **Resource limits obligatoires** via `HostConfig` bollard :
   ```rust
   HostConfig {
       memory: Some(512 * 1024 * 1024),  // 512 MB
       cpu_period: Some(100_000),
       cpu_quota: Some(50_000),           // 50% d'un core
       pids_limit: Some(256),
       ..Default::default()
   }
   ```

---

### 2.3 Gestionnaire Git (Rust / git2)

**Faisabilité : ✅ Élevée**

Le crate **git2** (bindings libgit2) couvre clone, fetch, checkout, diff, branch management. C'est la même lib sous-jacente que GitPython.

**Points à clarifier :**
- **Repos volumineux :** prévoir shallow clone (`git2::FetchOptions::depth(1)`) avec option de clone complet configurable par workspace.
- **Credentials SSH :** les clés injectées via `SecretRef` sont écrites dans un répertoire tmpfs (`/run/secrets/<workspace_id>/`) avec permissions `0600`, supprimées après le clone via un guard RAII Rust (`impl Drop`).
- **Branches éphémères pipeline :** créées automatiquement (`pipeline/<uid>/<timestamp>`), supprimées après merge/rejet par le worker.

---

### 2.4 API Control Plane (Rust / Axum)

**Faisabilité : ✅ Élevée**

Axum est le framework HTTP Rust le plus adapté : async natif tokio, extracteurs typés, middleware `tower`, excellente gestion des erreurs avec `thiserror`.

**Crates recommandés :**
| Besoin | Crate |
|--------|-------|
| HTTP framework | `axum` |
| Async runtime | `tokio` |
| PostgreSQL async | `sqlx` |
| Sérialisation | `serde` + `serde_json` |
| Validation | `validator` |
| Auth JWT | `jsonwebtoken` |
| Sessions cookie | `tower-sessions` + `axum-sessions` |
| Redis | `redis` (async feature) |
| HTTP client (LLM, webhooks) | `reqwest` |
| Docker API | `bollard` |
| Git | `git2` |
| Erreurs lib | `thiserror` |
| Erreurs bin | `anyhow` |
| Logs structurés | `tracing` + `tracing-subscriber` |
| OpenTelemetry | `opentelemetry` + `tracing-opentelemetry` |

**Points de vigilance :**

1. **Pas de macro-ORM :** SQLx valide les requêtes SQL à la compilation (`query!` macro). Aucune magie cachée, les requêtes sont lisibles et auditables.

2. **Pagination cursor-based** obligatoire sur tous les endpoints de liste dès le départ. Schema :
   ```json
   { "data": [...], "meta": { "next_cursor": "...", "has_more": true } }
   ```

3. **SSE temps réel :** Axum supporte nativement `axum::response::Sse`. Endpoint `GET /api/v1/workspaces/:uid/events` diffuse les transitions de statut et logs pipeline sans polling.

4. **Webhooks entrants :** vérification HMAC-SHA256 de la signature en middleware avant tout traitement. L'event est stocké dans `incoming_webhook_events` (TTL 7j) puis traité par un worker Redis Streams.

---

### 2.5 Dashboard (Next.js) + Web IDE Client

**Faisabilité : ✅ Élevée**

Next.js + shadcn/ui + Tailwind est une combinaison solide et accessible. La cible WCAG 2.1 AA est atteignable avec shadcn/ui (composants Radix sous-jacents, accessibles par défaut).

**Points pratiques dashboard :**
- Générer le client TypeScript depuis le schéma OpenAPI Rust (via `utoipa` + `openapi-typescript`) pour éviter toute désynchronisation types frontend/backend.
- SSE consommé via `EventSource` natif browser — pas de lib externe nécessaire.
- Mode "quick-start" au premier workspace : étapes 4 et 5 optionnelles, postposées, avec bandeau de progression non bloquant.

**Koda Web IDE (`apps/web-client/`) — plugin `koda-web-ide`**

Client web natif embarqué dans Koda, différenciateur de la plateforme par rapport à un simple hébergement de code-server.

| Composant | Technologie | Notes |
|-----------|------------|-------|
| Éditeur | **Monaco Editor** (Apache 2.0) | Même moteur que VS Code, support LSP, diff inline |
| File tree | React + Koda API | `GET /api/v1/workspaces/:uid/files` |
| Terminal | **xterm.js** + WebSocket | Via sozu `/[UID]/ide/terminal` → container PTY |
| Chat IA | SSE streaming | `POST /api/v1/workspaces/:uid/ai/chat` → AiProviderAdapter |
| Git panel | Koda API | diff, stage, commit, push sans quitter l'IDE |

**Flux chat IA → patch :**
```
1. Utilisateur décrit la modification en langage naturel
2. L'API envoie fichiers ouverts + prompt au LLM via AiProviderAdapter
3. Réponse streamée (SSE) affichée progressivement dans la sidebar
4. Si patch proposé → affiché en diff Monaco, appliqué sur clic utilisateur
5. Jamais de modification silencieuse du filesystem
```

**Risque faisabilité :** Monaco Editor dans un environnement `/[UID]/ide/` avec path stripping nécessite que les assets Monaco (workers JS) soient servis correctement sous ce chemin. Configurer le `publicPath` webpack/vite en conséquence dès le départ.

**Nouveaux endpoints API nécessaires :**
```
GET  /api/v1/workspaces/:uid/files          ?path=/src  → liste répertoire
GET  /api/v1/workspaces/:uid/files/content  ?path=...   → lit fichier
PUT  /api/v1/workspaces/:uid/files/content  ?path=...   → écrit fichier
POST /api/v1/workspaces/:uid/ai/chat                    → SSE streaming
GET  /api/v1/workspaces/:uid/ai/chat/:id                → historique session
```

---

### 2.6 Workers / Task Runner (Rust + Redis Streams)

**Faisabilité : ✅ Élevée**

Remplace Celery par des **workers Rust** consommant des **Redis Streams** avec consumer groups. Avantages :
- Même runtime tokio que l'API : pas de langage secondaire à maintenir.
- Redis Streams offre persistence native, replay, dead-letter intégré.
- Garantie "at-least-once" via `XACK` après traitement réussi.

**Architecture du worker :**
```
Redis Stream: jobs:workspace (create, start, stop, clone)
Redis Stream: jobs:pipeline  (build, lint, security)
Redis Stream: jobs:gateway   (expose, unexpose routes nginxify)
Redis Stream: jobs:dead_letter (échecs après 3 tentatives)
```

**Intégration LLM :** les pipelines IA appellent l'API LLM via `reqwest` + trait `AiProviderAdapter`. Pas de SDK LLM Rust officiel — l'API HTTP Anthropic/OpenAI est suffisante avec des structs `serde`.

**Point d'attention à 500k jobs/mois (horizon 2 ans) :** Redis Streams scale bien à ce volume sans changement d'architecture. La seule contrainte est la politique de rétention des streams (`MAXLEN`).

---

### 2.7 Authentification

**Faisabilité : ✅ Élevée**

Mêmes recommandations que l'analyse initiale, adaptées à Rust :

1. **TOTP via app (Google Authenticator, Authy)** préféré à l'OTP email pour le MFA — latence nulle, pas de dépendance SMTP. Crate : `totp-rs`. Email OTP en fallback uniquement.

2. **Rotation tokens M2M :** JWT court (15min) + refresh token hashé en DB (argon2 via crate `argon2`). Révocation via table `revoked_tokens` + check Redis pour hot path.

3. **Sessions cookie :** `HttpOnly` + `SameSite=Strict` + `Secure`. Stockage serveur-side dans Redis (pas JWT côté dashboard admin).

---

## 3. Risques transversaux

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R1 | Socket Docker accessible = vecteur d'escalade | Élevée (si non mitigé) | Critique | `docker-socket-proxy` en MVP, Sysbox à terme |
| R2 | WebSocket non supporté | Faible | Élevé | sozu gère HTTP Upgrade nativement |
| R3 | Clonage Git bloquant le worker | Modérée | Modéré | Shallow clone + timeout configurable + worker dédié |
| R4 | Absence de limites ressources container | Élevée | Élevé | Champs `NonZero` obligatoires dans HostConfig bollard |
| R5 | Dépendance LLM sans abstraction | Modérée | Modéré | Trait `AiProviderAdapter` à implémenter dès Phase 0 |
| R6 | Absence d'événements temps réel | Résolue | — | SSE Axum natif sur `/events` |
| R7 | Volumes Docker orphelins | Élevée (long terme) | Modéré | Job Rust cron (`jobs:gc`) planifié via Redis Streams |
| R8 | Multi-tenant data leak | Modérée | Critique | RLS PostgreSQL + filtre `organization_id` applicatif |
| R9 | Accès socket sozu non contrôlé | Faible | Élevé | Socket Unix local, accès limité au service gateway/ dans le même network Docker |

---

## 4. Lacunes dans les specs (toutes confirmées à corriger)

### 4.1 Entité manquante : `WorkspaceVolume`

Les volumes Docker (données persistantes) ne sont pas modélisés dans les specs. Proposition actée :
```
WorkspaceVolume {
  id, workspace_id, volume_name (Docker), size_mb, created_at,
  last_mounted_at, status: attached | detached | archived
}
```

### 4.2 Limites de ressources container

Ajouter à `Template` :
```
cpu_millicores INT NOT NULL DEFAULT 500,
memory_mb INT NOT NULL DEFAULT 512,
pid_limit INT NOT NULL DEFAULT 256,
storage_gb INT NOT NULL DEFAULT 10
```

### 4.3 Clarification Template vs Plugin

- **Template** : image Docker + runtime (ex: `ubuntu:22.04-node18`). Définit les ressources.
- **Plugin** : outil d'accès installé dans le container issu du Template (code-server, JetBrains, SSH). Génère les `ExposureRule`.
- Le container est instancié depuis l'image du **Template**, configuré pour le **Plugin**.

### 4.4 Queue d'events webhooks entrants

Table `incoming_webhook_events` (TTL 7j) pour stockage avant traitement. Signature HMAC-SHA256 vérifiée en middleware Axum avant insertion.

### 4.5 Health probe par plugin

Chaque `PluginDefinition` définit :
```
health_probe_path TEXT,     -- ex: "/healthz"
health_probe_port INT,      -- port interne à sonder
health_probe_timeout_s INT  -- ex: 60
```
Le worker poll jusqu'à succès ou timeout avant de passer le workspace en `running`.

---

## 5. Améliorations de l'architecture

### 5.1 Row Level Security PostgreSQL
RLS activé sur `workspaces`, `cicd_pipelines`, `tickets`, `audit_events`. Double filet contre les bugs de filtre applicatif.

### 5.2 OpenTelemetry dès Phase 0
`tracing` + `tracing-opentelemetry` dans chaque service Rust. Export OTLP vers Jaeger self-hosted. Coût d'ajout initial quasi nul, coût de retrofit élevé.

### 5.3 Bus d'events interne Redis Streams
Chaque transition d'état publie un event (`workspace.started`, `pipeline.completed`). Les workers et l'API SSE consomment depuis ces streams. Pas de polling DB.

### 5.4 SSE pour temps réel dashboard
`GET /api/v1/workspaces/:uid/events` (Axum `Sse`). Le dashboard s'abonne via `EventSource`. Pas de WebSocket côté API nécessaire.

### 5.5 Support `devcontainer.json`
Lire `.devcontainer/devcontainer.json` dans le repo cloné pour pré-remplir Template et Plugin. Compatibilité avec VS Code Dev Containers et milliers de repos existants.

### 5.6 `docker-socket-proxy` obligatoire MVP
Whitelist API Docker : `POST /containers/create`, `POST /containers/{id}/start`, `POST /containers/{id}/stop`, `DELETE /containers/{id}`, `GET /containers/{id}/json`, `POST /volumes/create`, `DELETE /volumes/{name}`. Rien d'autre.

### 5.7 Migrations sqlx-migrate — conventions renforcées
- Nommage : `YYYYMMDDHHMM_<objet>_<action>.sql`
- Chaque migration `.up.sql` accompagnée d'un `.down.sql` non-destructif
- Colonne NOT NULL : expand (nullable) → backfill → contract (NOT NULL) sur 3 déploiements
- DROP de colonne interdit avant 2 semaines de déprecation applicative

---

## 6. Nouvelles fonctionnalités proposées

### 6.1 Webhook Inbox par workspace *(port forwarding géré par nginxify)*

Chaque workspace reçoit une URL `https://domain.com/[UID]/webhook/[TOKEN]` qui capture les webhooks entrants et les affiche dans le dashboard avec corps complet. Pas de ngrok nécessaire. Stocké dans `incoming_webhook_events`.

### 6.2 Workspace Forking

Nouveau workspace depuis l'état courant d'un existant : copie du volume Docker, même branche Git, même PluginBinding. Cas d'usage : expérimentation sans risque, pair programming isolé.

### 6.3 Env Manager (Variables d'environnement UI)

Éditeur visuel des variables du workspace :
- Champs masqués pour secrets.
- Diff vs valeurs Template par défaut.
- Import `.env` local (parsé côté client, jamais envoyé brut).

### 6.4 Terminaux partagés (xterm.js + WebSocket)

Terminaux multiplexés WebSocket (nginx supporte nativement via nginxify), sessions nommées, lien partage temporaire. Fondamental pour pair programming sans IDE complet.

### 6.5 Snapshot chaud + Restauration

`docker pause` + copie volume → checkpoint en secondes. Rollback rapide avant opération risquée.

### 6.6 Pre-warming d'images

Job Rust cron quotidien qui pull les images Template les plus utilisées. Réduit le cold start de 30-120s à quelques secondes.

### 6.7 Pipeline IA : Review automatique de diff

Job Rust déclenché avant l'étape 7 (Revue). Appel LLM via `AiProviderAdapter` sur le diff Git. Produit :
- Résumé en langage naturel.
- Risques potentiels (sécurité, perf, breaking changes).
- Suggestions de refactoring.

Affiché en sidebar dans la vue Diff du dashboard.

### 6.8 Workspace Activity Feed

Timeline par workspace : clonage Git, démarrages/arrêts, exécutions pipeline, commits, tickets. Alimentée depuis `audit_events` + events Redis Streams.

### 6.9 Quotas par Organisation (`OrganizationQuota`)

```
max_concurrent_workspaces INT,
max_workspace_cpu_millicores INT,
max_workspace_memory_mb INT,
auto_hibernate_after_minutes INT
```
Vérifiés à la création workspace et appliqués par le worker.

### 6.10 CLI Koda (`koda connect <uid>`)

Client CLI Rust (binaire unique, distribuable) établissant un tunnel SSH vers le workspace. Pour les développeurs préférant leur terminal local. Distribué comme release GitHub.

---

## 7. Roadmap de faisabilité

### Phase 0 — Fondations (semaines 1-4)
- Monorepo initialisé (structure `apps/`, `services/`, `infra/`, `docs/`).
- PostgreSQL + sqlx-migrate + modèles de base (Workspace, User, Organization).
- Axum skeleton : auth session + OAuth, endpoints `/api/v1/auth/*`.
- Trait `AiProviderAdapter` (interface + implémentation Anthropic HTTP).
- Docker Compose dev : api, dashboard, web-client, db, redis, docker-socket-proxy, sozu.
- Service `gateway/` : client sozu-command-lib minimal (add/remove HttpFrontend).
- Pipeline Harness : lint + test + build image + push registry (toutes branches).
- Pipeline Harness prod : déploiement auto sur merge `main`.

### Phase 1 — Workspace minimal (semaines 5-10)
- Création workspace + UID.
- Clone Git asynchrone via git2 + worker Redis Streams.
- Lancement container bollard via docker-socket-proxy (avec resource limits).
- nginxify : creation/suppression ExposureRule via API.
- Dashboard : liste workspaces + statut en SSE.

### Phase 2 — Workspace complet (semaines 11-16)
- PluginBinding + health probe par plugin.
- ExposureRules dynamiques (create/update/delete via nginxify).
- Diff viewer (git2 + frontend).
- WorkspaceVolume lifecycle.
- TOTP MFA + tokens M2M.

### Phase 3 — Pipelines CI/CD (semaines 17-22)
- CiCdPipeline + AutomationTrigger.
- Workers Rust pour exécution pipeline dans container isolé.
- Webhook entrant HMAC + `IncomingWebhookEvent`.
- Webhook Inbox dashboard.

### Phase 4 — Sécurité & Observabilité (semaines 23-26)
- RBAC complet + `AuditEvent`.
- RLS PostgreSQL sur tables critiques.
- OpenTelemetry export OTLP + Sentry.
- Tests E2E Playwright (création workspace, revue diff, clôture).
- Review sécurité OWASP Top 10.

---

## 8. Checklist de cohérence finale

| Critère | Statut | Note |
|---------|--------|------|
| Stack compatible self-hosted VPS | ✅ | Docker Compose + sozu + PostgreSQL |
| Contrat API aligné entités métier | ✅ | Pagination cursor + SSE à implémenter |
| Sécurité multi-tenant | ✅ | RLS PostgreSQL + filtre organization_id |
| Migrations rollback-safe | ✅ | sqlx-migrate + expand/contract |
| WCAG 2.1 AA dashboard | ✅ | shadcn/ui (Radix) accessible par défaut |
| Budget hébergement plausible | ✅ | 75-180 EUR/mois réaliste pour MVP |
| Isolation container | ⚠️ | docker-socket-proxy obligatoire + resource limits bollard |
| WebSocket gateway | ✅ | sozu supporte HTTP Upgrade nativement |
| Port forwarding HTTP | ✅ | sozu HttpFrontend + strip_prefix |
| Port forwarding TCP (SSH, PgSQL) | ✅ | sozu TcpFrontend sur ports dédiés |
| TLS / Let's Encrypt | ✅ | sozu intégré, aucune gestion applicative |
| Volumes persistants formalisés | ✅ | Entité `WorkspaceVolume` ajoutée |
| Health checks workspace | ✅ | Probe définie dans `PluginDefinition` |
| Auth sozu ↔ Koda | ✅ | Socket Unix local, pas d'API réseau exposée |
| LLM abstraction | ⚠️ | Trait `AiProviderAdapter` à implémenter Phase 0 |
| CI/CD Harness | ✅ | Mirror GitHub → Harness, build → registry, deploy prod auto sur main |
| Web IDE (Monaco) | ⚠️ | publicPath webpack à configurer sous `/[UID]/ide/` dès le départ |
| Opérations fichiers IDE | ✅ | Via Koda API uniquement, middleware auth Axum obligatoire |
