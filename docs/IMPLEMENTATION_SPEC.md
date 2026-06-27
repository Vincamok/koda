# Spécification d'implémentation — Koda

> Référence de développement : fonctions à implémenter par module.
> Mise à jour à chaque nouvelle version.

---

## `apps/api/` — Axum API Control Plane

### `src/main.rs`
- `main()` : initialisation tokio runtime, chargement config, démarrage serveur Axum

### `src/config.rs`
- `AppConfig` : struct de configuration (DB, Redis, sozu socket, AI provider, ports)
- `load_config()` : lecture depuis env vars + `config/platform.config.yaml`

### `src/db.rs`
- `create_pool()` : pool SQLx PostgreSQL avec timeout et max connections
- `run_migrations()` : exécution sqlx-migrate au démarrage

### `src/error.rs`
- `AppError` : enum thiserror couvrant tous les types d'erreur applicatifs
- `impl IntoResponse for AppError` : sérialisation JSON `{"error": {"code", "message", "request_id"}}`

### `src/middleware/`
- `auth.rs` / `require_auth()` : extracteur Axum vérifiant session cookie, retourne `AuthUser`
- `auth.rs` / `require_role(role)` : vérifie RBAC sur l'organisation courante
- `rate_limit.rs` / `rate_limit_layer()` : tower middleware, limite par IP + par user_id
- `request_id.rs` / `request_id_layer()` : injecte `X-Request-Id` dans chaque requête

### `src/handlers/auth.rs`
- `post_register()` : inscription email/password, hash argon2, envoi email de vérification
- `post_login()` : vérification credentials, création session Redis, cookie HttpOnly
- `post_logout()` : révocation session Redis, clear cookie
- `get_me()` : retourne `AuthUser` depuis session
- `get_oauth_authorize(provider)` : redirect OAuth (Google, GitHub)
- `get_oauth_callback(provider, code)` : échange code → token, upsert user, création session
- `post_verify_email(token)` : validation token d'email
- `post_otp_verify(code)` : vérification TOTP (totp-rs)
- `post_otp_setup()` : génération secret TOTP + QR code
- `post_token_create()` : création token M2M (JWT court + refresh token hashé en DB)
- `post_token_refresh()` : rotation token M2M
- `post_token_revoke()` : révocation RFC 7009

### `src/handlers/workspaces.rs`
- `post_workspace()` : création workspace (UID UUID v4, statut `created`), vérifie quota org
- `get_workspace(uid)` : retourne workspace + état courant
- `get_workspaces()` : liste paginée (cursor-based) filtrée par `organization_id`
- `post_workspace_start(uid)` : déclenche job `workspace.start` → Redis Streams
- `post_workspace_stop(uid)` : déclenche job `workspace.stop` → Redis Streams
- `delete_workspace(uid)` : déclenche job `workspace.destroy` → Redis Streams
- `get_workspace_diff(uid)` : retourne diff Git (via git-manager service)
- `get_workspace_events(uid)` : SSE stream des transitions de statut + logs
- `get_workspace_files(uid, path)` : liste répertoire dans le volume workspace
- `get_workspace_file_content(uid, path)` : lit contenu d'un fichier
- `put_workspace_file_content(uid, path)` : écrit contenu d'un fichier

### `src/handlers/git.rs`
- `get_workspace_git(uid)` : retourne `WorkspaceGitConfig` actif
- `put_workspace_git(uid)` : configure/met à jour git (url, branche, SecretRef), déclenche clone
- `get_workspace_git_status(uid)` : statut clone + dernière erreur si failed

### `src/handlers/plugins.rs`
- `get_plugins()` : liste `PluginDefinition` du catalogue
- `post_workspace_plugin(uid)` : sélectionne plugin actif, déclenche provisioning
- `get_workspace_plugin(uid)` : retourne `WorkspacePluginBinding` actif + statut

### `src/handlers/pipelines.rs`
- `get_workspace_pipelines(uid)` : liste pipelines du workspace
- `post_workspace_pipeline(uid)` : crée un `CiCdPipeline` (type, config YAML)
- `get_workspace_pipeline(uid, pid)` : détail pipeline + dernière exécution
- `post_pipeline_run(uid, pid)` : déclenche exécution manuelle → Redis Streams
- `get_pipeline_run(uid, pid, rid)` : résultat d'une exécution (logs, statut)

### `src/handlers/triggers.rs`
- `get_workspace_triggers(uid)` : liste `AutomationTrigger` du workspace
- `post_workspace_trigger(uid)` : crée trigger (on_push | schedule | manual)
- `patch_workspace_trigger(uid, tid)` : active/désactive ou modifie config trigger
- `delete_workspace_trigger(uid, tid)` : supprime trigger

### `src/handlers/webhooks.rs`
- `post_webhook_receive(uid, token)` : reçoit webhook entrant, vérifie HMAC, stocke `IncomingWebhookEvent`
- `get_workspace_webhook_inbox(uid)` : liste events reçus (paginé, TTL 7j)
- `get_webhook_event(uid, eid)` : détail d'un event (headers, body, timestamp)

### `src/handlers/ai.rs`
- `post_workspace_ai_chat(uid)` : reçoit message + fichiers contexte, stream SSE réponse LLM
  - Charge les `WorkspaceMCPBinding` actifs, injecte tool definitions dans le prompt
  - Si LLM retourne `tool_call` : publie dans `jobs:mcp`, attend résultat Redis, continue stream
- `get_workspace_ai_chat_history(uid, cid)` : historique d'une session de chat

### `src/handlers/mcp.rs`
- `get_mcp_connectors()` : liste `MCPConnectorDefinition` du catalogue (depuis `mcp_connector_definitions`)
- `get_workspace_mcp_bindings(uid)` : connecteurs actifs du workspace
- `post_workspace_mcp_binding(uid)` : active un connecteur (config + SecretRefs), crée `WorkspaceMCPBinding`
- `delete_workspace_mcp_binding(uid, bid)` : désactive un connecteur, révoque SecretRefs
- `get_user_mcp_bindings()` : connecteurs MCP personnels de l'utilisateur connecté
- `post_user_mcp_binding()` : ajoute un connecteur MCP personnel (UserMCPBinding)
- `delete_user_mcp_binding(bid)` : supprime un connecteur MCP personnel

### `src/handlers/teams.rs`
- `get_org_teams(oid)` : liste les Teams de l'organisation
- `post_org_team(oid)` : crée un Team
- `patch_org_team(oid, tid)` : renomme / modifie description
- `delete_org_team(oid, tid)` : supprime un Team (vérifie qu'il n'a plus de membres)
- `get_team_members(oid, tid)` : liste les membres du Team avec leur rôle
- `post_team_member(oid, tid)` : ajoute un membre au Team avec rôle
- `patch_team_member(oid, tid, uid)` : change le rôle (lead | developer | reviewer | viewer)
- `delete_team_member(oid, tid, uid)` : retire un membre
- `get_team_projects(oid, tid)` : projets accessibles au Team
- `post_team_project(oid, tid)` : donne accès à un projet
- `delete_team_project(oid, tid, pid)` : retire l'accès à un projet
- `get_team_quota(oid, tid)` : retourne `TeamQuota`
- `put_team_quota(oid, tid)` : configure le quota du Team

### `src/handlers/personal.rs`
- `get_personal_space()` : retourne le PersonalSpace de l'utilisateur connecté
- `get_personal_file(path)` : lit un fichier `.personal/<path>`
- `put_personal_file(path)` : écrit un fichier `.personal/<path>` (crée le volume si absent)
- `delete_personal_file(path)` : supprime un fichier
- `get_personal_snippets()` : liste les snippets personnels
- `post_personal_snippet()` : crée un snippet
- `patch_personal_snippet(sid)` : modifie un snippet
- `delete_personal_snippet(sid)` : supprime un snippet
- `get_workspace_personal_note(uid)` : note personnelle de l'utilisateur sur ce workspace
- `put_workspace_personal_note(uid)` : crée/met à jour la note

### `src/handlers/security.rs`
- `get_workspace_security_reports(uid)` : liste les SecurityReport du workspace
- `get_security_report(uid, rid)` : détail d'un rapport + VulnerabilityFinding[]
- `get_org_security_policy(oid)` : politique sécurité de l'organisation
- `put_org_security_policy(oid)` : configure `SecurityPolicy` (owner/admin uniquement)

### `src/handlers/tickets.rs`
- `get_workspace_tickets(uid)` : liste `TicketRecord` du workspace
- `post_workspace_ticket(uid)` : crée ticket (titre, description, priorité)
- `get_workspace_ticket(uid, tid)` : détail ticket
- `patch_workspace_ticket(uid, tid)` : met à jour statut/description
- `get_workspace_ticket_export(uid, tid)` : export Markdown

### `src/handlers/jobs.rs`
- `post_job()` : crée job générique (analyse IA, export)
- `get_job(id)` : statut et résultat du job

### `src/handlers/orgs.rs`
- `post_organization()` : crée organisation
- `get_organization(oid)` : détail organisation
- `post_org_member(oid)` : invite utilisateur avec rôle
- `patch_org_member(oid, uid)` : change rôle d'un membre
- `delete_org_member(oid, uid)` : retire membre
- `get_org_quota(oid)` : retourne `OrganizationQuota`
- `put_org_quota(oid)` : configure quotas (owner/admin uniquement)

### `src/models/` — SQLx structs
- `workspace.rs` : Workspace, WorkspaceGitConfig, WorkspaceVolume, WorkspacePluginBinding, WorkspaceShare
- `plugin.rs` : PluginDefinition, ExposureRule
- `pipeline.rs` : CiCdPipeline, AutomationTrigger, PipelineRun
- `user.rs` : User, Organization, Membership
- `team.rs` : Team, TeamMembership, TeamProjectAccess, TeamQuota
- `personal.rs` : PersonalSpace, PersonalSnippet
- `job.rs` : Job, IncomingWebhookEvent
- `ticket.rs` : TicketRecord
- `secret.rs` : SecretRef
- `audit.rs` : AuditEvent
- `quota.rs` : OrganizationQuota
- `mcp.rs` : MCPConnectorDefinition, WorkspaceMCPBinding, UserMCPBinding
- `security.rs` : SecurityReport, VulnerabilityFinding, SecurityPolicy

### `src/ai/`
- `provider.rs` / `trait AiProviderAdapter` : `chat_stream(messages, context) -> Stream<String>`
- `anthropic.rs` / `AnthropicAdapter` : implémentation reqwest + SSE Anthropic API
- `openai.rs` / `OpenAiAdapter` : implémentation reqwest + SSE OpenAI API (optionnel)

---

## `services/orchestrator/` — Cycle de vie containers

### `src/main.rs`
- `main()` : démarre consommateur Redis Streams `jobs:workspace`

### `src/docker.rs`
- `DockerClient` : wrapper bollard avec docker-socket-proxy
- `create_workspace_container(workspace, plugin, volume) -> ContainerId`
  - Applique HostConfig : `memory`, `cpu_period`, `cpu_quota`, `pids_limit`
  - Monte le volume workspace
  - Configure les variables d'environnement (SecretRef résolus)
- `start_container(id)`
- `stop_container(id, timeout_secs)`
- `remove_container(id)`
- `inspect_container(id) -> ContainerState`
- `get_container_ip(id) -> IpAddr`

### `src/volume.rs`
- `create_volume(name) -> VolumeInfo`
- `remove_volume(name)`
- `archive_volume(name)` : docker cp vers objet storage (futur)

### `src/probe.rs`
- `wait_for_healthy(host, port, path, timeout) -> Result<()>`
  - Poll `GET http://{host}:{port}{path}` jusqu'à 200 ou timeout
  - Met à jour `WorkspacePluginBinding.status` en DB à chaque étape

### `src/quota.rs`
- `check_org_quota(org_id) -> Result<()>` : vérifie `max_concurrent_workspaces`
- `check_resource_quota(org_id, template) -> Result<()>` : vérifie CPU/RAM cumulé

### `src/network.rs`
- `create_workspace_networks(ws_uid) -> NetworkIds` : crée `koda-ws-<uid>-internal` + `koda-ws-<uid>-services`
- `remove_workspace_networks(ws_uid)` : supprime les réseaux du workspace
- `attach_container_to_networks(container_id, plugin_def)` : attache selon `PluginDefinition.network_policy`
- `ensure_egress_network()` : crée `koda-egress` s'il n'existe pas (réseau partagé internet)
- `get_container_ip(container_id, network) -> IpAddr` : IP du container sur un réseau donné

### `src/personal.rs`
- `ensure_personal_volume(user_uid) -> VolumeName` : crée `koda-personal-<user-uid>` si absent
- `mount_personal_volume(container_id, user_uid)` : monte le volume en read-only sur `/home/koda/.personal/`
- `symlink_shell_configs(container_id)` : lie `.personal/shell/.zshrc` → `/home/koda/.zshrc` etc.
- `symlink_git_config(container_id)` : lie `.personal/git/.gitconfig` → `/home/koda/.gitconfig`
- `run_startup_script(container_id)` : exécute `.personal/workspace/startup.sh` si présent

### `src/jobs.rs`
- `handle_workspace_start(job)` :
  1. `check_org_quota` + `check_team_quota`
  2. `create_workspace_networks`
  3. `create_workspace_container` (avec labels `koda.*` obligatoires)
  4. `mount_personal_volume` + symlinks configs personnelles
  5. `start` → `probe` → `update ExposureRule` → `SSE event`
- `handle_workspace_stop(job)` : stop_container → detach_volume → remove ExposureRule
- `handle_workspace_destroy(job)` : stop → remove_container → `remove_workspace_networks` → archive_volume → cleanup DB

---

## `services/worker/` — Redis Streams consumer

### `src/main.rs`
- `main()` : spawn consumers pour chaque stream (workspace, pipeline, gateway, gc)

### `src/consumer.rs`
- `StreamConsumer` : struct tokio + redis XREADGROUP + XACK
- `consume(stream, group, handler)` : boucle avec retry exponentiel
- `move_to_dead_letter(job)` : après 3 échecs → `jobs:dead_letter`

### `src/jobs/pipeline.rs`
- `run_pipeline(pipeline, workspace)` :
  - Crée branche éphémère `pipeline/<uid>/<timestamp>` (git2)
  - Lance container pipeline isolé (bollard)
  - Exécute commande selon type (build/lint/security_scan)
  - Capture stdout/stderr → stocke résultat en DB
  - Met à jour statut `CiCdPipeline`
  - Supprime branche éphémère après exécution

### `src/jobs/ai_review.rs`
- `run_diff_review(workspace)` :
  - Récupère diff Git (git2)
  - Appelle `AiProviderAdapter.chat_stream()` avec diff comme contexte
  - Stocke résumé + risques en DB liés au workspace

### `src/jobs/security.rs`
- `run_secret_scan(workspace, pipeline_run_id)` : scanne le code (regex patterns + entropie Shannon) → `SecurityReport`
- `run_sast(workspace, pipeline_run_id)` : appelle LLM sécurité dédié (system prompt OWASP) sur le diff → `VulnerabilityFinding[]`
- `run_dependency_scan(workspace, pipeline_run_id)` : exécute cargo audit / npm audit / pip-audit dans container isolé
- `run_image_scan(image_name, pipeline_run_id)` : lance Trivy/Grype sur l'image → findings
- `check_security_policy(org_id, report_id) -> bool` : vérifie si `SecurityPolicy.min_severity_to_block` est atteint

### `src/jobs/gc.rs`
- `collect_orphan_volumes()` : liste volumes Docker sans workspace actif → archive/supprime
- `cleanup_dead_letter()` : log + alerte sur jobs dead-letter
- `prewarm_images()` : docker pull des images Template populaires

### `src/cron.rs`
- `start_cron_scheduler()` : lance jobs planifiés (gc quotidien, prewarm quotidien)
- `evaluate_schedule_triggers()` : évalue les `AutomationTrigger` de type `schedule`

---

## `services/git-manager/` — Opérations Git

### `src/clone.rs`
- `clone_repository(url, branch, dest, credentials) -> Result<()>`
  - Shallow clone configurable (`--depth 1`)
  - Credentials SSH via tmpfs + Drop guard
  - Mise à jour `clone_status` en DB à chaque étape

### `src/branch.rs`
- `create_ephemeral_branch(repo, name) -> BranchRef`
- `delete_ephemeral_branch(repo, name)`
- `checkout_branch(repo, name)`

### `src/diff.rs`
- `get_diff(repo) -> DiffResult` : diff entre HEAD et working tree
- `get_diff_between(repo, base, head) -> DiffResult`
- `format_diff_json(diff) -> Vec<FileDiff>` : sérialisation pour l'API

### `src/ops.rs`
- `stage_files(repo, paths)` : git add
- `commit(repo, message, author) -> CommitHash`
- `push(repo, remote, branch, credentials)`
- `pull(repo, remote, branch, credentials)`

### `src/fs.rs`
- `list_directory(repo_path, rel_path) -> Vec<FileEntry>` : pour l'API web-client
- `read_file(repo_path, rel_path) -> Bytes`
- `write_file(repo_path, rel_path, content)`

---

## `services/gateway/` — Client sozu

### `src/main.rs`
- `main()` : écoute events `jobs:gateway` depuis Redis Streams

### `src/sozu_client.rs`
- `SozuClient` : connexion au socket Unix sozu
- `add_http_frontend(uid, service, path_prefix, backend_addr)`
- `remove_http_frontend(uid, service)`
- `add_tcp_frontend(host_port, backend_addr)`
- `remove_tcp_frontend(host_port)`
- `add_backend(cluster_id, addr)`
- `remove_backend(cluster_id, addr)`
- `list_frontends() -> Vec<Frontend>` : état courant du proxy

### `src/port_allocator.rs`
- `allocate_port(protocol) -> u16` : trouve le prochain port libre dans la plage
- `release_port(protocol, port)` : libère le port en DB
- `is_port_available(port) -> bool`

### `src/jobs.rs`
- `handle_expose(rule: ExposureRule)` : ajoute route sozu + met à jour DB
- `handle_unexpose(rule: ExposureRule)` : supprime route sozu + libère port si TCP

---

## `apps/web-client/` — IDE web natif (Monaco + IA)

### `src/components/Editor.tsx`
- `MonacoEditor` : wrapper `@monaco-editor/react`, publicPath auto, config LSP de base
- `DiffEditor` : affichage diff avant/après pour les patches IA

### `src/components/FileTree.tsx`
- `FileTree` : arbre de fichiers depuis `GET /api/v1/workspaces/:uid/files`
- `FileTreeNode` : noeud cliquable, icônes par type, drag-and-drop (futur)
- `NewFileDialog` : création fichier/dossier inline

### `src/components/Terminal.tsx`
- `Terminal` : xterm.js monté sur WebSocket `/[UID]/ide/terminal`
- `useTerminalSocket(uid)` : hook gestion connexion + reconnexion automatique

### `src/components/AiSidebar.tsx`
- `AiSidebar` : panneau chat IA
- `ChatMessage` : message utilisateur ou réponse IA (Markdown rendu)
- `PatchProposal` : affichage diff Monaco + bouton "Appliquer"
- `useAiChat(uid)` : hook SSE streaming vers `/api/v1/workspaces/:uid/ai/chat`

### `src/components/GitPanel.tsx`
- `GitPanel` : onglet Git (diff, fichiers modifiés, staged, commit)
- `FileDiff` : affichage unified diff par fichier
- `CommitForm` : message de commit + bouton push

### `src/components/StatusBar.tsx`
- `StatusBar` : barre inférieure (branche Git, statut connexion, raccourcis)

### `src/lib/device.ts`
- `detectDeviceMode() -> 'full-ide' | 'tablet-ide' | 'mobile-view'` : viewport + touch detection
- `useDeviceMode()` : hook React retournant le mode courant (réactif au resize)

### `src/components/PromptLevelSelector.tsx`
- `PromptLevelSelector` : sélecteur du niveau de prompt (nano|quick|standard|deep|agent)
- `usePromptLevel()` : hook stockant le niveau choisi, persisté dans PersonalSpace `editor/settings.json`
- `AgentConfirmDialog` : modal de confirmation avant chaque action du niveau 5 (écriture fichier, exécution)

### `src/components/PersonalPanel.tsx`
- `PersonalPanel` : panneau "Mon espace" (icône profil dans la barre latérale)
- `PersonalFileEditor` : Monaco intégré pour éditer les fichiers `.personal/` (ai/, shell/, git/, editor/, workspace/, notes/)
- `SnippetsManager` : liste/création/édition des snippets personnels par langage
- `PersonalMCPList` : connecteurs MCP personnels (UserMCPBinding) actifs sur tous les workspaces
- `WorkspaceNote` : note markdown personnelle sur le workspace courant (non partagée)
- `usePersonalSpace()` : hook `GET|PUT /api/v1/users/me/personal/files/*`

### `src/lib/ai-context.ts`
- `buildAiContext(uid, level, openFiles) -> AiContext` : construit le contexte LLM selon le niveau
  - Merge `CLAUDE.md` workspace + `.personal/ai/CLAUDE.md` + `.personal/ai/context.md`
  - Filtre `.env`, `*.key`, `*.pem` avant tout envoi
  - Niveaux 1-2 : fichier courant uniquement
  - Niveau 3 : fichiers ouverts + arbre (noms)
  - Niveaux 4-5 : workspace complet + Git + CI + MCP tools
- `sanitizeForLlm(content) -> string` : supprime patterns secrets (tokens, clés, mots de passe)
- `detectPromptInjection(content) -> boolean` : détecte tentatives d'injection dans le contenu utilisateur

### `src/components/MCPPanel.tsx`
- `MCPPanel` : panneau liste connecteurs disponibles + bindings actifs du workspace
- `ConnectorCard` : carte connecteur (nom, catégorie, icône, outils disponibles, toggle actif)
- `ConnectorConfigForm` : formulaire dynamique généré depuis `configFields` (inclut champs secret masqués)
- `ActiveBindingList` : liste des connecteurs activés avec statut + bouton détacher
- `useMCPConnectors(uid)` : hook `GET /api/v1/mcp/connectors` + `GET /api/v1/workspaces/:uid/mcp/bindings`

### `src/lib/api.ts`
- `listFiles(uid, path)`, `readFile(uid, path)`, `writeFile(uid, path, content)`
- `sendChatMessage(uid, message, context)` → EventSource
- `stageFiles(uid, paths)`, `commit(uid, message)`, `push(uid)`
- `listMCPConnectors()` : `GET /api/v1/mcp/connectors`
- `createMCPBinding(uid, connectorId, config, secretRefs)` : `POST /api/v1/workspaces/:uid/mcp/bindings`
- `deleteMCPBinding(uid, bindingId)` : `DELETE /api/v1/workspaces/:uid/mcp/bindings/:bindingId`

---

## `apps/dashboard/` — Interface d'administration

### `src/app/` — Pages Next.js
- `page.tsx` : redirect vers `/dashboard` si authentifié, sinon `/login`
- `login/page.tsx` : formulaire login + OAuth buttons
- `register/page.tsx` : formulaire inscription + validation email
- `dashboard/page.tsx` : vue d'ensemble (stats org, workspaces actifs, jobs récents)
- `workspaces/page.tsx` : liste workspaces (cards + statuts temps réel SSE)
- `workspaces/new/page.tsx` : wizard création workspace (étapes 1-3 obligatoires)
- `workspaces/[uid]/page.tsx` : détail workspace + onglets (Git, Plugin, Pipelines, Tickets)
- `workspaces/[uid]/review/page.tsx` : vue diff + panneau review IA
- `settings/page.tsx` : profil utilisateur, TOTP, tokens M2M
- `settings/org/page.tsx` : gestion organisation, membres, quotas (owner/admin)

### `src/components/workspace/`
- `WorkspaceCard.tsx` : carte workspace (UID, statut badge, plugin actif, actions)
- `WorkspaceStatus.tsx` : badge animé selon statut (created/running/reviewing/closed)
- `WorkspaceWizard.tsx` : stepper création workspace (étapes 1-5)
- `GitConfigForm.tsx` : formulaire URL repo + branche + SecretRef
- `PluginSelector.tsx` : grille de sélection plugin avec preview
- `DiffViewer.tsx` : diff Git avec syntax highlight + actions (valider/rejeter)
- `ActivityFeed.tsx` : timeline chronologique des événements workspace

### `src/components/pipeline/`
- `PipelinePanel.tsx` : liste pipelines + statuts + bouton run manuel
- `PipelineRun.tsx` : détail exécution (logs streaming, durée, statut)
- `TriggerForm.tsx` : formulaire création trigger (on_push | schedule | manual)

### `src/components/ui/`
- `ThemeSwitcher.tsx` : dropdown sélection skin (preview miniature)
- `StatusBadge.tsx` : badge générique avec couleur selon statut
- `SseStatus.tsx` : indicateur connexion SSE (reconnexion automatique)
- `PaginatedList.tsx` : liste avec cursor pagination et scroll infini

### `src/lib/`
- `api-client.ts` : client HTTP généré depuis OpenAPI (tous les appels API)
- `sse.ts` : `useServerSentEvents(url)` hook avec reconnexion exponentielle
- `auth.ts` : helpers session, logout, vérification token

---

## `services/mcp-gateway/` — Proxy MCP (Rust)

### `src/main.rs`
- `main()` : init tracing, charge `Config`, démarre `SessionManager`

### `src/config.rs`
- `Config { redis_url, database_url }` : depuis env vars `REDIS_URL`, `DATABASE_URL`

### `src/session.rs`
- `SessionManager` : consommateur Redis Streams `jobs:mcp` (XREADGROUP, XACK)
- `handle_message(fields)` : résoud SecretRef → route vers connecteur → TODO: publie résultat Redis

### `src/secret.rs`
- `SecretResolver.resolve_binding_config(binding_id)` : charge config `WorkspaceMCPBinding` depuis DB, résoud SecretRef (TODO: Vault integration)

### `src/connectors/mod.rs`
- `McpResult { content: Value, is_error: bool }`
- `trait McpConnector` : `id()`, `list_tools()`, `call_tool(tool, args, config)`, `read_resource(uri, config)`
- `ConnectorRegistry` : HashMap, auto-enregistre les 6 connecteurs built-in

### `src/connectors/github.rs`
- `GitHubConnector` : tools `list_issues`, `get_pr`, `search_code`, `create_issue`, `comment_pr`
- `gh_client(token)` : headers Authorization/Accept/X-GitHub-Api-Version/User-Agent
- `uri_to_api_url(uri)` : convertit `github://owner/repo/blob/branch/path` → URL API

### `src/connectors/jira.rs`
- `JiraConnector` : tools `search_issues` (JQL), `get_issue`, `create_issue`, `transition_issue`
- `jira_client(email, token)` : Basic Auth base64

### `src/connectors/notion.rs`
- `NotionConnector` : tools `search`, `get_page`, `create_page`, `append_block`
- Header `Notion-Version: 2022-06-28`

### `src/connectors/postgres.rs`
- `PostgresConnector` : tools `query`, `list_tables`, `describe_table`
- `is_write_query(sql)` : bloque INSERT/UPDATE/DELETE/DROP/etc. si `readonly=true`
- `row_to_json(row)` : sérialise `PgRow` → `serde_json::Value`

### `src/connectors/slack.rs`
- `SlackConnector` : tools `slack_post_message`, `slack_search_messages`, `slack_list_channels`
- `slack_client(token)` : header `Authorization: Bearer {bot_token}`

### `src/connectors/http.rs`
- `HttpConnector` : tools `http_get`, `http_post`, `http_patch`, `http_delete`
- `build_client(config)` : supporte auth `none | bearer | apikey-header | basic`

### `src/proxy.rs`
- Stub — futur serveur HTTP/SSE protocole MCP natif pour connecteurs stdio (v0.4.0+)

---

## `packages/mcp-connectors/` — Registre TypeScript MCP

### `src/types.ts`
- `MCPConnectorDefinition` : id, name, description, version, category, capabilities, configFields, tools, resourceTemplates
- `WorkspaceMCPBinding` : id, workspaceId, connectorId, config, secretRefIds, enabled
- `ConfigField` : key, label, type (text/password/url/select/boolean/number/textarea), required, secret
- `MCPTool`, `MCPResource`, `MCPResourceTemplate`, `MCPPrompt`, `JsonSchema`
- `MCPCallRequest` / `MCPCallResponse`

### `src/registry.ts`
- `MCPConnectorRegistry` : observable Map + Set listeners
- `register()`, `unregister()`, `get()`, `list()`, `listByCategory()`, `listByCapability()`, `search()`
- Singleton `mcpRegistry`

### `src/connectors/`
- `github.ts` : 5 tools + 3 resource templates, configFields `[gh_token]`
- `jira.ts` : 4 tools (JQL search, get, create, transition), configFields `[base_url, email, api_token]`
- `notion.ts` : 4 tools, configFields `[notion_token]`
- `postgres.ts` : 3 tools + readonly guard, configFields `[connection_string, readonly, max_rows]`
- `slack.ts` : 3 tools, configFields `[bot_token]`
- `http.ts` : 4 tools + 4 auth modes, configFields `[base_url, auth_type, ...]`

### `src/index.ts`
- Auto-enregistre les 6 connecteurs built-in dans `mcpRegistry` au chargement du module

---

## `packages/themes/` — Système de thèmes

### `src/types.ts`
- `Skin` : interface complète (id, name, layout, colorScheme, typography, spacing)
- `LayoutVariant` : `'sidebar-left' | 'sidebar-right' | 'top-nav' | 'minimal'`
- `ColorScheme` : record de CSS custom properties

### `src/themes/`
- `default.ts` : Koda Default (dark, sidebar-left, densité standard)
- `minimal.ts` : Koda Minimal (dark, top-nav, épuré)
- `pro.ts` : Koda Pro (dark, sidebar-left + activity bar, dense)
- `light.ts` : Koda Light (light, sidebar-left, radius doux)

### `src/registry.ts`
- `ThemeRegistry` : observable Map + Set listeners
- `register()`, `unregister()`, `get()`, `getOrDefault()`, `list()`, `extend(baseId, overrides)`
- `loadManifest(manifest)` : résoud `extends`, construit Skin complet, enregistre
- `loadFromUrl(url)` : fetch JSON array de `SkinManifest`, charge tous en batch
- Singleton `themeRegistry`

### `src/ThemeProvider.tsx`
- `ThemeProvider` : Context React, applique CSS variables + classe layout sur `<html>`
- `useTheme()` : hook retournant skin courant + `setSkin(id)` + `availableSkins` (réactif au registre)
- Persistance : `localStorage` côté client, DB `user.preferred_skin` côté serveur
- S'abonne à `themeRegistry.onChange()` pour mettre à jour `availableSkins` dynamiquement

### `src/ThemeSwitcher.tsx`
- `ThemeSwitcher` : radiogroup de boutons avec miniatures `SkinPreview`
- `SkinPreview` : mini-schéma du layout généré depuis les tokens CSS du skin (sidebar, editor, IA, statusbar)

### `src/css/base.css`
- Grid CSS pour `[data-layout='sidebar-left']` et `[data-layout='top-nav']`
- Named areas : titlebar, sidebar, editor, terminal, ai-sidebar, statusbar
- `top-nav` : sidebar et AI sidebar en overlays (CSS transform animé)
- Density variables `--spacing-1/2/3` par `data-density`
- Focus-visible WCAG 2.1 AA avec `--primary` outline

---

## `infra/migrations/` — sqlx-migrate

### Ordre d'exécution
1. `202600010000_organizations_create.sql`
2. `202600010001_users_create.sql`
3. `202600010002_memberships_create.sql`
4. `202600010003_organization_quotas_create.sql`
5. `202600010004_projects_create.sql`
6. `202600010005_templates_create.sql`
7. `202600010006_workspaces_create.sql`
8. `202600010007_workspace_git_configs_create.sql`
9. `202600010008_workspace_volumes_create.sql`
10. `202600010009_plugin_definitions_create.sql`
11. `202600010010_workspace_plugin_bindings_create.sql`
12. `202600010011_exposure_rules_create.sql`
13. `202600010012_cicd_pipelines_create.sql`
14. `202600010013_automation_triggers_create.sql`
15. `202600010014_incoming_webhook_events_create.sql`
16. `202600010015_jobs_create.sql`
17. `202600010016_ticket_records_create.sql`
18. `202600010017_secret_refs_create.sql`
19. `202600010018_audit_events_create.sql`
20. `202600010019_workspace_shares_create.sql`
21. `202600010020_teams_create.sql`
22. `202600010021_team_memberships_create.sql`
23. `202600010022_team_project_access_create.sql`
24. `202600010023_team_quotas_create.sql`
25. `202600010024_personal_spaces_create.sql`
26. `202600010025_personal_snippets_create.sql`
27. `202600010026_workspace_shares_create.sql`
28. `202600010027_mcp_connector_definitions_create.sql`
29. `202600010028_workspace_mcp_bindings_create.sql`
30. `202600010029_user_mcp_bindings_create.sql`
31. `202600010030_security_policies_create.sql`
32. `202600010031_security_reports_create.sql`
33. `202600010032_vulnerability_findings_create.sql`
34. `202600010033_enable_rls.sql` : activation RLS sur tables critiques
