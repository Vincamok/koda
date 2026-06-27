# MEMORY.md — Koda

## Contexte projet
Koda est une plateforme de gestion d'environnements de développement à la demande, auto-hébergée sur VPS. Chaque workspace est un conteneur Docker isolé, accessible via une URL unique `domain.com/[UID]/[service]` gérée par **sozu** (reverse-proxy Rust avec API de configuration dynamique).

## Décisions architecturales actées

| Décision | Choix retenu | Alternative écartée |
|----------|-------------|---------------------|
| Gateway | **sozu** (reverse-proxy Rust, TLS + HTTP/2 + TCP natifs) | nginxify, Traefik, proxy custom |
| Routing HTTP | Path-based `[UID]/[service]` via sozu HttpFrontend | — |
| Routing TCP | Port-based via sozu TcpFrontend (SSH, PostgreSQL) | HAProxy |
| Port forwarding HTTP | sozu HttpFrontend + StripPrefix | — |
| Port forwarding TCP | sozu TcpFrontend (port dédié par tunnel) | — |
| Backend API | **Axum** (Rust) + SQLx + tokio | FastAPI (Python) |
| Workers / Task Runner | **Rust** (tokio + Redis Streams) | Celery (Python) |
| Docker SDK (Rust) | **bollard** (Docker API async) | docker-py |
| Git (Rust) | **git2** (libgit2 bindings) | shell git |
| LLM integration | **reqwest** + HTTP direct (AiProviderAdapter trait) | SDK Python |
| Frontend dashboard | **Next.js** + TypeScript + shadcn/ui + Tailwind | SvelteKit |
| Web IDE client | **Monaco Editor** + xterm.js + chat IA sidebar | code-server seul |
| MCP connecteurs | **@koda/mcp-connectors** (registre + 6 built-in) | Intégration directe |
| MCP gateway | **services/mcp-gateway/** (Rust, trait McpConnector) | Appels HTTP directs |
| Thèmes — extensibilité | **ThemeRegistry** + SkinManifest (chargement dynamique) | Record statique |
| BDD | PostgreSQL | — |
| Migrations | **sqlx-migrate** (fichiers SQL versionnés) | Alembic |
| Isolation socket Docker | docker-socket-proxy (whitelist API) | Socket brut |
| Queue broker | Redis Streams (consumer groups) | RabbitMQ |
| CI/CD | **Harness self-hosted** (mirror GitHub) | GitHub Actions |
| Registry images | **Harness Artifact Registry** | Docker Hub, GHCR |
| Déploiement prod | Auto sur merge `main` via Harness pipeline | Manuel |
| RBAC | **Org + Teams + WorkspaceShare** (3 couches) | ProjectMembership flat |
| Config service | **`config/default.yaml` par service** + `.env.example` + figment | Config centrale |
| Réseaux Docker | **Multi-réseau par workspace** (`internal`, `services`, `koda-egress`) | Réseau unique |
| Espace personnel | **PersonalSpace** (volume Docker personnel par user + fichiers config) | Settings en DB seuls |
| Sécurité intégrée | **Scans dans CI/CD** + LLM sécurité configurable (SecurityAiConfig) + SecurityPolicy | Post-prod uniquement |
| LLM sécurité | **SecurityAiConfig** par org (provider + model + system_prompt overridable) | Instance partagée avec chat IA |
| secret_scan | **Règles built-in** (entropy + regex) + **ScanRule custom** (org + workspace) | Trufflehog/Gitleaks externe |
| image_scan déclencheur | **ImageScanTrigger configurable** dans SecurityPolicy (OnBuild \| OnLaunch \| Both) | Toujours au lancement |
| Proxy trust | **`TRUSTED_PROXY_CIDRS`** par service + `axum-client-ip` | Trust aveugle headers |
| SecretRef stockage | **DB colonne chiffrée AES-256-GCM** (clé dans env) + Docker env inject pour secrets runtime container | HashiCorp Vault (dépendance externe) |
| Auth providers | **Google + GitHub + Authentik** (OIDC générique — extensible) | Auth custom seule |
| Types TS partagés | **`packages/shared-types/`** (types communs dashboard+web-client+admin) + **`packages/api-client/`** (client HTTP généré depuis OpenAPI) | Types copiés par app |
| Interface admin | **`apps/admin/`** (Next.js séparé) + rôle `super_admin` (au-dessus de `owner`) + `KodaInstance` pour multi-instances | Section dans dashboard |
| Workspace clôture | **Libre** — aucun blocage sur la phase `reviewing` | Revue obligatoire |
| TCP port ranges | **SSH `2200-2999`**, **PostgreSQL `5400-5499`** (réservé dans sozu, stocké dans `ExposureRule.host_port`) | — |

## Environnements

### Dev
- Code sur **GitHub** (`main` + branches `feature/*`, `fix/*`)
- Mirror automatique GitHub → **Harness self-hosted** (effet miroir)
- Harness build les images Docker sur chaque push/PR
- Images taguées `sha-<commit>` et poussées vers le **Harness registry**
- 2 développeurs, pas de client externe, pas de staging séparé

### Prod
- Déploiement déclenché automatiquement par Harness sur merge `main`
- Pull des images depuis le Harness registry (tag `sha-<commit>` du merge)
- Rollback = redéployer le tag précédent depuis Harness

### Pipeline CI (Harness) — toutes branches
1. `cargo test --workspace` + `cargo clippy`
2. `npm run test` + `npm run lint` (dashboard + web-client)
3. Build images Docker multi-stage
4. Push vers Harness registry (tag `sha-<commit>`)
5. **Sur merge `main` uniquement :** déploiement prod automatique

## Plugin catalogue

| Plugin | Type | Description |
|--------|------|-------------|
| `koda-web-ide` | web | **Client web natif Koda** — Monaco Editor + file tree + terminal xterm.js + chat IA sidebar. Endpoint `/[UID]/ide` |
| `code-server` | web | VS Code complet dans le navigateur. Endpoint `/[UID]/vscode` |
| `ssh` | tcp | Accès SSH direct via tunnel sozu TcpFrontend (port dédié) |
| `jupyter` | web | JupyterLab pour workspaces data/ML. Endpoint `/[UID]/jupyter` |

## Koda Web IDE (`koda-web-ide`)

Client web natif, différenciateur de la plateforme. Vit dans `apps/web-client/`.

**Composants :**
- **Monaco Editor** (Apache 2.0) — moteur VS Code, support syntaxe, LSP
- **File tree** — lecture/écriture fichiers via `GET|PUT /api/v1/workspaces/:uid/files`
- **Terminal** — xterm.js + WebSocket sozu → container PTY
- **Chat IA sidebar** — streaming SSE via `POST /api/v1/workspaces/:uid/ai/chat`
  - Lit les fichiers ouverts comme contexte
  - Propose des patches appliqués en un clic
  - Utilise `AiProviderAdapter` (Anthropic par défaut)
- **Git panel** — diff, stage, commit, push via Koda API
- **Panel MCP** — activation/config des connecteurs MCP par workspace
- **Panel PersonalSpace** — édition des fichiers `.personal/` de l'utilisateur

**Modes selon le device :**
| Mode | Device détecté | Composants actifs |
|------|---------------|-------------------|
| `full-ide` | PC (>1280px, no touch) | Monaco + FileTree + Terminal + AiSidebar + GitPanel + MCP |
| `tablet-ide` | Tablette paysage | Monaco + AiSidebar + FileTree overlay + sans terminal |
| `mobile-view` | Mobile / tablette portrait | AiSidebar chat + FileViewer (lecture seule) |

**5 niveaux de prompts IA :**

| Niveau | Déclencheur | Contexte envoyé | Sécurité |
|--------|-------------|-----------------|----------|
| 1 — Nano | Frappe auto | ±50 lignes curseur | Filtre secrets |
| 2 — Quick | Sélection + `⌘K` | Sélection + fichier courant | Filtre credentials |
| 3 — Standard | Chat sidebar (défaut) | Fichiers ouverts + arbre (noms) | `.env` jamais inclus |
| 4 — Deep | Bouton "Analyse complète" | Workspace complet + Git + CI + MCP | Rate limited + AuditEvent + confirmation |
| 5 — Agent | Mode Agent explicite | Workspace + outils (lecture/écriture/exécution) | Confirmation avant chaque action + kill switch + sandbox |

**Règles transversales IA :**
- `.env`, `*.key`, `*.pem`, secrets résolus → jamais dans aucun prompt
- Détection prompt injection sur contenus utilisateur (commentaires, docstrings) avant injection LLM
- Niveaux 4-5 → AuditEvent obligatoire
- Patches toujours affichés en diff Monaco avant application

**Endpoints API requis :**
```
GET  /api/v1/workspaces/:uid/files?path=/src
GET  /api/v1/workspaces/:uid/files/content?path=
PUT  /api/v1/workspaces/:uid/files/content?path=
POST /api/v1/workspaces/:uid/ai/chat             # SSE streaming, body inclut prompt_level
GET  /api/v1/workspaces/:uid/ai/chat/:id
GET  /api/v1/mcp/connectors
POST /api/v1/workspaces/:uid/mcp/bindings
DELETE /api/v1/workspaces/:uid/mcp/bindings/:bid
GET  /api/v1/users/me/personal                   # PersonalSpace de l'utilisateur connecté
PUT  /api/v1/users/me/personal/files/:path       # Édition fichiers personal
GET  /api/v1/users/me/mcp/bindings               # MCP personnels
```

## RBAC — Droits utilisateurs

### Trois couches indépendantes

**Org-level :**
| Rôle | Droits |
|------|--------|
| `owner` | Tout — facturation, suppression org, transfert ownership |
| `admin` | Membres, teams, quotas, projets — pas facturation |
| `member` | Utilise la plateforme dans les limites quota |

**Team-level :**
| Rôle | Droits |
|------|--------|
| `lead` | Gère le team, invite, gère les projets du team |
| `developer` | Crée/gère ses workspaces, push, accès terminal |
| `reviewer` | Lit workspaces, commente diffs, approuve reviews — pas terminal |
| `viewer` | Lecture seule — pas de terminal ni écriture |

**Workspace-level (ad-hoc via WorkspaceShare) :**
`editor` · `reviewer` · `viewer` — durée limitée, hors-org possible

### Hiérarchie
```
Organization (tenant, facturation, quotas globaux)
  └── Team (groupe d'accès, quota propre)
       └── Project → Workspace
  └── Member (appartient à l'org, peut être dans 0..n Teams)
```

### Entités Teams
- `Team(id, org_id, name, description)`
- `TeamMembership(team_id, user_id, role, granted_by)`
- `TeamProjectAccess(team_id, project_id)`
- `TeamQuota(team_id, max_workspaces, max_cpu, max_ram)` — sous-ensemble du quota org

## Intégration sozu

Le service `services/gateway/` est un client Rust de sozu via `sozu-command-lib`.
Il traduit les `ExposureRule` Koda en commandes sozu, sans jamais éditer de config fichier.

**Routes HTTP :**
```
[UID]/ide     → HttpFrontend { path_prefix: "/[UID]/ide",    strip_prefix: true } → container:4000
[UID]/vscode  → HttpFrontend { path_prefix: "/[UID]/vscode", strip_prefix: true } → container:8080
```

**Routes TCP (port dédié par tunnel) :**
```
:2201 → TcpFrontend → container SSH :22
:5433 → TcpFrontend → container PostgreSQL :5432
```
Plages réservées : SSH `2200-2999`, PostgreSQL `5400-5499`. Stockées dans `ExposureRule.host_port`.

## Docker — Réseaux et containers

### Stratégie réseau

```
koda-platform (bridge, services plateforme)
  ├── api, orchestrator, worker, git-manager, gateway, mcp-gateway
  └── sozu  ← seul à router vers les réseaux workspace

koda-ws-<uid>-internal (bridge isolé, pas d'internet)
  ├── postgres container, redis container, services internes

koda-ws-<uid>-services (bridge workspace)
  ├── web-app container (aussi sur internal)
  └── koda-web-ide, code-server

koda-egress (réseau partagé, sortie internet contrôlée)
  └── containers nécessitant internet (npm install, git clone, pip...)
```

sozu accède aux containers workspace via IP Docker directe — pas de réseau commun nécessaire.

### Nommage containers et réseaux
- Container workspace : `koda-<binding-uid>` (binding-uid = UUID du WorkspacePluginBinding)
- Internal host dans sozu : `svc-<binding-uid>` (alias réseau)
- Réseau internal : `koda-ws-<workspace-uid>-internal`
- Réseau services : `koda-ws-<workspace-uid>-services`

### Labels obligatoires sur tous containers éphémères
```
koda.managed=true
koda.type=workspace|pipeline|plugin
koda.workspace_id=<uid>
koda.org_id=<org_id>
koda.binding_id=<binding_uid>
```
Le GC utilise ces labels pour retrouver les containers orphelins même après crash de l'orchestrateur.

### PluginDefinition — réseau requis
```yaml
network_policy:
  networks: [internal, egress]  # quels réseaux attacher
  expose:
    - port: 4000
      protocol: http
```

### Images Docker

**Services plateforme :**
- Un `Dockerfile` par service dans son dossier (`services/orchestrator/Dockerfile`)
- Image builder partagée `koda-rust-base` (layer mis en cache, builds rapides)
- Multi-stage obligatoire : `builder` (cargo build) → `runtime` (distroless/alpine ≈20 MB)
- `HEALTHCHECK` obligatoire dans chaque Dockerfile

**Images workspace (pré-buildées) :**
```
infra/docker/workspace-images/
  base/Dockerfile.ubuntu-base     # layer partagé (outils communs, user koda uid=1000, sshd)
  Dockerfile.ubuntu-node          # Node.js 20/22
  Dockerfile.ubuntu-python        # Python 3.11/3.12
  Dockerfile.ubuntu-go            # Go 1.22+
  Dockerfile.ubuntu-rust          # Rust + cargo
```
Pré-warmées par worker cron — jamais de pull à chaud au lancement workspace.

**Sécurité runtime :**
- User non-root `koda` (uid 1000) dans les images workspace
- `no-new-privileges` flag dans HostConfig
- Seccomp profile whitelist syscalls
- Filesystem read-only (sauf volume workspace monté en rw)

### docker-compose
```
infra/docker/
  docker-compose.yml           # Services plateforme (toujours)
  docker-compose.override.yml  # Dev : ports exposés, hot-reload (gitignored)
  docker-compose.prod.yml      # Prod : resource limits, pas de port exposé
```

## Configuration par service

Chaque service est autonome. Priorité (highest wins) :
1. Variables d'environnement (prod Harness, Docker `-e`)
2. `.env` local
3. `config/default.yaml` — valeurs par défaut

```
services/orchestrator/
  config/default.yaml     # valeurs par défaut, commité
  .env.example            # template vars à surcharger, commité
  .env                    # overrides locaux, gitignored
  Dockerfile
  Cargo.toml

apps/api/
  config/default.yaml
  .env.example
  Dockerfile
```

Crate Rust : **`figment`** (merge YAML + env + .env, typage fort).

## Services derrière reverse proxy

Chaque service Rust doit :
- Lire `TRUSTED_PROXY_CIDRS` depuis sa config (ex: `["10.0.0.0/8", "172.16.0.0/12"]`)
- Extraire l'IP client via `axum-client-ip` (header `X-Forwarded-For` trusté uniquement depuis ces CIDRs)
- Utiliser `APP_BASE_URL` (config) comme seule source pour les URLs absolues (OAuth, emails, redirects)
- Ne jamais inférer le proto/host depuis les headers sans validation proxy

## PersonalSpace — espace personnel utilisateur

Chaque utilisateur dispose d'un espace personnel portable qui voyage avec lui dans tous ses workspaces.

**Volume Docker** : `koda-personal-<user-uid>` — monté en lecture seule dans chaque workspace à `/home/koda/.personal/`

**Structure des fichiers :**
```
.personal/
├── ai/
│   ├── CLAUDE.md              # Instructions IA (style, préférences, langue)
│   ├── context.md             # Background : stack maîtrisé, domaines, expérience
│   ├── coding-style.md        # Conventions personnelles, patterns préférés/à éviter
│   ├── review-checklist.md    # Checklist de review envoyée à l'IA avant validation
│   └── prompts/               # Prompts favoris sauvegardés par niveau
│       ├── quick.md
│       ├── standard.md
│       └── agent.md
│
├── editor/
│   ├── settings.json          # Monaco : font, tab size, word wrap, minimap, rulers
│   ├── keybindings.json       # Raccourcis personnels
│   ├── themes-order.json      # Ordre de préférence des skins
│   └── snippets/              # Snippets par langage
│       ├── rust.json
│       ├── typescript.json
│       ├── python.json
│       └── sql.json
│
├── shell/
│   ├── .zshrc                 # Config shell principale
│   ├── .aliases               # Aliases personnels
│   ├── .functions             # Fonctions shell réutilisables
│   ├── .exports               # Variables d'env non-secrètes (PATH, EDITOR...)
│   └── scripts/               # Scripts utilitaires personnels
│
├── git/
│   ├── .gitconfig             # Identité, aliases, signing (SecretRef pour clé GPG)
│   ├── .gitignore_global      # Patterns ignorés globalement
│   └── .gitmessage            # Template de message de commit
│
├── workspace/
│   ├── .editorconfig          # Préférences de formatage (fallback si pas de config projet)
│   ├── env_defaults.json      # Variables injectées dans chaque workspace au démarrage
│   └── startup.sh             # Script exécuté à chaque ouverture de workspace
│
└── notes/
    ├── README.md              # Wiki / notes personnelles
    ├── bookmarks.md           # Ressources, liens utiles
    └── workspace-notes/       # Notes par workspace (non partagées avec l'équipe)
        └── <workspace-uid>.md
```
**Note :** Les bindings MCP personnels (`UserMCPBinding`) sont stockés en DB uniquement — pas de fichier dans le PersonalSpace (les SecretRef ne peuvent pas être en fichier).

**Fusion avec le workspace :**
| Fichier | Priorité |
|---------|----------|
| `ai/CLAUDE.md` + fichiers ai/ | Workspace + personnel (additifs) |
| `.editorconfig` | Projet > personnel |
| `.gitconfig` | Personnel (identité) + `.git/config` projet (remote) |
| `env_defaults.json` | Personnel injecté, workspace peut surcharger |
| Snippets editor | Additifs (les deux disponibles) |
| MCP bindings | Additifs (workspace + personnel) |

**Règle de sécurité :** les fichiers `ai/` du PersonalSpace ne sont jamais loggués ni transmis hors du contexte LLM.

## Interface Administration (`apps/admin/`)

App Next.js dédiée, séparée du dashboard utilisateur. Accessible sur `/admin`, restreinte au rôle `super_admin` (distinct de `owner` org). Peut être déployée sur un réseau interne uniquement en prod.

### Modules

| Module | Contenu |
|--------|---------|
| **Tableau de bord global** | Workspaces actifs, containers, CPU/RAM, santé des services (api, orchestrator, worker, sozu, Redis, PG), alertes (dead-letter, orphelins, quotas dépassés) |
| **Organisations** | CRUD, suspension/réactivation, quotas (`max_workspaces`, `max_cpu_cores`, `max_ram_gb`, `max_storage_gb`, `max_members`) |
| **Utilisateurs** | Vue globale toutes orgs, affectations, désactivation compte, reset MFA, impersonation (AuditEvent obligatoire) |
| **IA & pré-prompts** | Provider global par défaut, override par org, system prompt global + override par org (SecurityAiConfig), templates de prompts par niveau éditables |
| **Logs & audit** | Vue unifiée `AuditEvent` (filtres : org, user, action, date), jobs Redis (streams + dead-letter), VulnerabilityFindings agrégés, export CSV/JSON |
| **Infrastructure** | Containers Docker actifs (`koda.managed=true`), routes sozu actives, taille DB, migrations, GC manuel orphelins |
| **Sécurité** | ScanRule built-in (lecture seule), SecurityPolicy par org, derniers rapports de scan |
| **Multi-instances** | Vue de toutes les `KodaInstance` connectées, métriques agrégées, affectation org → instance, basculement d'une org |

### Rôle `super_admin`

Rôle plateforme (pas org-scoped). Créé en bootstrap via variable d'env `BOOTSTRAP_SUPER_ADMIN_EMAIL`. Peut impersonner n'importe quel utilisateur ou org — chaque action d'impersonation génère un `AuditEvent`.

### Multi-instances (`KodaInstance`)

Permet à un panel admin central de piloter plusieurs déploiements Koda :

```
KodaInstance(id, name, base_url, api_token_ref: SecretRef, region, status, last_seen_at)
OrgInstanceAffinity(org_id, instance_id)  — quelle org sur quelle instance
```

Chaque instance expose `GET /api/v1/admin/health` authentifié par token M2M. Le panel central agrège les métriques et peut déclencher une migration d'org d'une instance à l'autre.

## Sécurité intégrée dans les projets

### Scans CI/CD

| Étape pipeline | Description | Déclencheur |
|---------------|-------------|-------------|
| `secret_scan` | Détection credentials dans le code (regex + entropie) | Chaque commit |
| `sast` | LLM sécurité dédié + règles statiques (OWASP Top 10 par langage) | PR / push |
| `dependency_scan` | `cargo audit`, `npm audit`, `pip-audit` | Quotidien + PR |
| `image_scan` | Trivy/Grype sur l'image workspace avant lancement | Build image |

### LLM sécurité — provider et modèle configurables

Réutilise le trait `AiProviderAdapter` mais comme instance distincte avec `SecurityAiConfig` :

```rust
pub struct SecurityAiConfig {
    pub provider:      String,  // "anthropic", "openai", "ollama", ...
    pub model:         String,  // modèle choisi (ex: claude-haiku-4-5 pour rapidité)
    pub system_prompt: String,  // OWASP built-in ou override org en DB
    pub max_tokens:    u32,
}
```

- Chaque org configure son propre LLM sécurité dans `SecurityPolicy.security_ai_config`
- Default : même provider que le chat IA général, modèle léger, system prompt OWASP built-in
- `system_prompt` en DB → l'admin peut affiner les règles sans redéployer
- Format de réponse structuré imposé : `JSON {findings: [{file, line, severity, category, description, fix}]}`

### `secret_scan` — règles natives + règles custom évolutives

Pattern Open/Closed (même approche que MCP et thèmes) :

**Règles built-in** (non supprimables, toujours actives) :
- Shannon entropy > seuil sur strings > N chars → token probable
- Regex : AWS keys, GitHub tokens, `-----BEGIN * KEY-----`, `sk_live_*`, `xoxb-*` (Slack), JWT `eyJ*`, URLs avec credentials...

**Règles custom** (org + workspace) :
```rust
pub struct ScanRule {
    pub id:        Uuid,
    pub name:      String,
    pub rule_type: RuleType,   // Regex | Entropy | Composite (entropy ET regex)
    pub pattern:   String,
    pub severity:  Severity,
    pub enabled:   bool,
}
```

Hiérarchie d'application :
```
built-in rules (toujours)
  + OrgScanRule (patterns métier org — ex: format clé API interne)
    + WorkspaceScanRule (patterns propres à un workspace)
```

Entité : `ScanRule(id, org_id nullable, workspace_id nullable, rule_type, pattern, severity, enabled)`

### `image_scan` — déclencheur configurable

```rust
pub enum ImageScanTrigger {
    OnBuild,    // dans Harness CI à chaque build d'image (défaut)
    OnLaunch,   // dans l'orchestrateur à chaque lancement workspace
    Both,       // recommandé en prod
    Disabled,
}
```

Stocké dans `SecurityPolicy.image_scan_trigger` — configurable par org.

### Nouvelles entités sécurité
- `SecurityReport` : workspace_id, pipeline_run_id, triggered_by, score_global
- `VulnerabilityFinding` : report_id, severity, category, file, line, description, fix_suggestion
- `SecurityPolicy` : org_id, required_scans[], min_severity_to_block, security_ai_config, image_scan_trigger
- `ScanRule` : org_id nullable, workspace_id nullable, rule_type, pattern, severity, enabled

### Intégration dans le cycle workspace
```
Workspace: reviewing
  ├── diff review IA (existant)
  ├── SecurityReport généré automatiquement
  └── bloqué si SecurityPolicy.min_severity_to_block atteint
```

## Architecture MCP

### Flux d'un tool call MCP depuis le web-client
```
Chat IA (web-client)
  → POST /api/v1/workspaces/:uid/ai/chat  (message + connecteurs actifs workspace + personnels)
  → API Axum injecte les tool definitions MCP dans le prompt LLM
  → LLM retourne un tool_call { connector_id, tool_name, arguments }
  → API publie dans Redis Stream jobs:mcp
  → mcp-gateway consomme, résoud les SecretRef, appelle le connecteur Rust
  → Résultat publié dans Redis → SSE vers le client
```

### Extensibilité MCP
**Connecteurs built-in (Rust + TypeScript) :** github, notion, postgres, slack, jira, http
**Ajout d'un connecteur custom :**
1. TypeScript : implémenter `MCPConnectorDefinition` + `mcpRegistry.register()`
2. Rust : implémenter `trait McpConnector` + `registry.register()`
3. Pas de modification du code existant — pattern Open/Closed

### Extensibilité des thèmes
**Thèmes built-in :** default, minimal, pro, light
**Ajout d'un thème custom :**
- Via code : `themeRegistry.register(mySkin)`
- Via JSON (DB/marketplace) : `themeRegistry.loadManifest(manifest)` avec `extends: 'default'`
- Via URL : `themeRegistry.loadFromUrl(url)` (marketplace futur)

## Entités métier clés
- `Organization` : tenant. Statuts : actif | suspended
- `User` : compte utilisateur. 1 PersonalSpace par User
- `Membership` : User ↔ Organization, rôle org (owner | admin | member)
- `Team` : groupe d'accès dans une org
- `TeamMembership` : User ↔ Team, rôle team (lead | developer | reviewer | viewer)
- `TeamProjectAccess` : Team ↔ Project
- `TeamQuota` : limites par team (sous-ensemble OrganizationQuota)
- `PersonalSpace` : espace personnel 1:1 avec User, lié à un volume Docker
- `PersonalSnippet` : snippet de code personnel (user_id, language, name, content)
- `UserMCPBinding` : connecteur MCP personnel (user-scoped, vs WorkspaceMCPBinding workspace-scoped)
- `Workspace` : instance identifiée par UID immuable. Statuts : `created → configuring → running → reviewing → closing → closed`
- `WorkspaceGitConfig` : config Git du workspace (1 actif max), clone_status : `pending → cloning → ready | failed`
- `WorkspacePluginBinding` : plugin actif déclenchant le provisioning container. Statuts : `installing → ready | failed`. UID = identité du container Docker
- `ExposureRule` : mapping route ↔ container. Champs : `protocol (http|tcp)`, `public_path | host_port`, `internal_host`, `internal_port`, `strip_prefix`
- `WorkspaceVolume` : volume Docker persistant lié au workspace
- `WorkspaceShare` : partage ad-hoc workspace, rôle (editor|reviewer|viewer), durée limitée
- `CiCdPipeline` : pipeline build/lint/security. Statuts : `idle → running → passed | failed`
- `AutomationTrigger` : déclencheur on_push | schedule | manual
- `IncomingWebhookEvent` : event webhook entrant stocké (TTL 7j) avant traitement worker
- `OrganizationQuota` : limites de ressources par organisation
- `AuditEvent` : traçabilité de toutes les actions critiques
- `MCPConnectorDefinition` : catalogue des connecteurs MCP disponibles (built-in + custom)
- `WorkspaceMCPBinding` : connecteur actif dans un workspace avec config + SecretRefs
- `SecurityReport` : résultat d'un scan sécurité lié à un workspace/pipeline
- `VulnerabilityFinding` : finding individuel d'un SecurityReport
- `SecurityPolicy` : org_id, required_scans[], min_severity_to_block, security_ai_config (JSON), image_scan_trigger
- `ScanRule` : règle de détection custom (org_id|workspace_id, rule_type Regex|Entropy|Composite, pattern, severity, enabled)
- `KodaInstance` : instance Koda enregistrée dans le panel admin central (base_url, api_token_ref, region, status)
- `OrgInstanceAffinity` : affectation org → instance Koda
- `TicketRecord` : lien entre un workspace et un ticket externe (Jira, Linear, GitHub Issues) — backlog post-v1.0.0

## Contraintes non-négociables
- Pas de Docker-in-Docker (DinD)
- Path Stripping HTTP obligatoire via sozu (l'app ne voit jamais le préfixe /[UID]/)
- organization_id obligatoire sur toutes les entités exposées (+ RLS PostgreSQL)
- Aucun secret stocké en clair (SecretRef uniquement) — y compris configs MCP et clés GPG
- Limites CPU/RAM/PID obligatoires sur chaque conteneur workspace (bollard HostConfig)
- Opérations fichiers web-client : toujours via Koda API, jamais accès direct au volume
- Containers workspace : user non-root (uid 1000), no-new-privileges, seccomp profile
- Headers proxy (`X-Forwarded-*`) : trustés uniquement depuis `TRUSTED_PROXY_CIDRS`
- Fichiers PersonalSpace `ai/` : jamais loggués ni transmis hors contexte LLM

## Risques principaux
1. Socket Docker = vecteur root → mitigé par docker-socket-proxy
2. WebSocket → sozu supporte HTTP upgrade nativement
3. TLS → sozu gère la terminaison TLS + renouvellement Let's Encrypt
4. Absence de health probe par plugin → mécanisme probe défini dans PluginDefinition
5. Volumes orphelins → garbage collector planifié (worker Rust cron)
6. LLM sans abstraction → AiProviderAdapter trait dès Phase 0
7. Accès fichiers web-client non authentifié → middleware auth Axum sur tous les endpoints `/files`
8. IP spoofing via headers proxy → TRUSTED_PROXY_CIDRS validé sur chaque service
9. Prolifération containers orphelins → labels `koda.*` + GC par labels
10. PersonalSpace non isolé → volume monté read-only, shell configs jamais en root

## Arborescence projet
```
apps/
  dashboard/        # Next.js + TypeScript + shadcn/ui  (gestion org — users, workspaces)
  web-client/       # React + Monaco Editor + xterm.js  (IDE in-browser)
  admin/            # Next.js — panel super_admin (quotas, logs, multi-instances, infra)
  api/              # Rust — Axum + SQLx + tokio
    config/
      default.yaml
    .env.example
services/
  orchestrator/     # Rust — cycle de vie containers (bollard)
    config/default.yaml
    .env.example
  worker/           # Rust — Redis Streams consumer (jobs async)
    config/default.yaml
    .env.example
  git-manager/      # Rust — clone/branches éphémères (git2)
    config/default.yaml
    .env.example
  gateway/          # Rust — client sozu-command-lib (gestion ExposureRules)
    config/default.yaml
    .env.example
  mcp-gateway/      # Rust — proxy MCP (trait McpConnector + 6 connecteurs built-in)
    config/default.yaml
    .env.example
packages/
  shared-types/     # Types TypeScript partagés (dashboard + web-client + admin)
  api-client/       # Client HTTP TypeScript généré depuis l'OpenAPI Koda (fetch typed)
  mcp-connectors/   # Définitions + registre des connecteurs MCP (TypeScript)
  themes/           # ThemeRegistry + 4 skins + SkinManifest
infra/
  docker/
    docker-compose.yml           # Services plateforme
    docker-compose.override.yml  # Dev overrides (gitignored)
    docker-compose.prod.yml      # Prod overrides
    workspace-images/            # Images workspace pré-buildées
      base/Dockerfile.ubuntu-base
      Dockerfile.ubuntu-node
      Dockerfile.ubuntu-python
      Dockerfile.ubuntu-go
      Dockerfile.ubuntu-rust
  harness/          # Pipelines Harness (YAML)
  migrations/       # sqlx-migrate — fichiers SQL versionnés
docs/               # Architecture + schémas Mermaid
```

## Commandes utiles
```bash
sudo docker compose up -d          # Lancer l'environnement de dev
sqlx migrate run                   # Appliquer les migrations
cargo test --workspace             # Tests unitaires Rust
cargo build --release              # Build production
sozuctl status                     # État du proxy sozu
docker network ls | grep koda      # Voir les réseaux workspace actifs
docker ps --filter label=koda.managed=true  # Voir tous les containers workspace
```
