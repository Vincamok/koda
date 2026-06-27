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
- `get_workspace_ai_chat_history(uid, cid)` : historique d'une session de chat

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
- `workspace.rs` : Workspace, WorkspaceGitConfig, WorkspaceVolume, WorkspacePluginBinding
- `plugin.rs` : PluginDefinition, ExposureRule
- `pipeline.rs` : CiCdPipeline, AutomationTrigger, PipelineRun
- `user.rs` : User, Organization, Membership, WorkspaceShare
- `job.rs` : Job, IncomingWebhookEvent
- `ticket.rs` : TicketRecord
- `secret.rs` : SecretRef
- `audit.rs` : AuditEvent
- `quota.rs` : OrganizationQuota

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

### `src/jobs.rs`
- `handle_workspace_start(job)` : create_container → start → probe → update ExposureRule → SSE event
- `handle_workspace_stop(job)` : stop_container → detach_volume → remove ExposureRule
- `handle_workspace_destroy(job)` : stop → remove_container → archive_volume → cleanup DB

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

### `src/lib/api.ts`
- `listFiles(uid, path)`, `readFile(uid, path)`, `writeFile(uid, path, content)`
- `sendChatMessage(uid, message, context)` → EventSource
- `stageFiles(uid, paths)`, `commit(uid, message)`, `push(uid)`

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

### `src/ThemeProvider.tsx`
- `ThemeProvider` : Context React, applique CSS variables + classe layout sur `<html>`
- `useTheme()` : hook retournant skin courant + `setSkin(id)`
- Persistance : `localStorage` côté client, DB `user.preferred_skin` côté serveur

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
21. `202600010020_enable_rls.sql` : activation RLS sur tables critiques
