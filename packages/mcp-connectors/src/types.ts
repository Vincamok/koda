/**
 * Types MCP (Model Context Protocol) pour les connecteurs Koda.
 * Suit la spec MCP officielle (https://modelcontextprotocol.io) tout en
 * ajoutant les abstractions nécessaires pour le système de plugins Koda.
 */

// ─── Spec MCP core ──────────────────────────────────────────────────────────

export interface MCPTool {
  name: string
  description: string
  inputSchema: JsonSchema
}

export interface MCPResource {
  uri: string
  name: string
  description?: string
  mimeType?: string
}

export interface MCPResourceTemplate {
  uriTemplate: string          // RFC 6570 URI template
  name: string
  description?: string
  mimeType?: string
}

export interface MCPPrompt {
  name: string
  description?: string
  arguments?: Array<{ name: string; description?: string; required?: boolean }>
}

export type JsonSchema = {
  type: 'object' | 'string' | 'number' | 'boolean' | 'array'
  properties?: Record<string, JsonSchema>
  required?: string[]
  description?: string
  enum?: unknown[]
  items?: JsonSchema
  [key: string]: unknown
}

// ─── Système de connecteurs Koda ────────────────────────────────────────────

export type ConnectorCategory =
  | 'vcs'           // GitHub, GitLab, Bitbucket
  | 'project'       // Jira, Linear, Notion, Trello
  | 'communication' // Slack, Discord, Teams
  | 'database'      // PostgreSQL, MySQL, MongoDB, Redis
  | 'cloud'         // AWS, GCP, Azure, Vercel
  | 'monitoring'    // Datadog, Sentry, Grafana
  | 'ai'            // OpenAI, Anthropic, Hugging Face
  | 'http'          // Generic REST API
  | 'custom'        // User-defined

export type ConnectorCapability =
  | 'read-context'    // Fournit du contexte lu au LLM (resources)
  | 'execute-tools'   // Le LLM peut déclencher des actions (tools)
  | 'subscribe-events'// Notifications push (webhook / polling)
  | 'search'          // Recherche sémantique dans la source

/**
 * Champ de configuration affiché dans le formulaire de setup du connecteur.
 */
export interface ConfigField {
  key: string
  label: string
  description?: string
  type: 'text' | 'password' | 'url' | 'select' | 'boolean' | 'number' | 'textarea'
  required: boolean
  placeholder?: string
  options?: Array<{ label: string; value: string }>  // pour type: 'select'
  secret?: boolean   // sera stocké via SecretRef, jamais en clair
  defaultValue?: unknown
}

/**
 * Définition statique d'un connecteur MCP.
 * Enregistrée dans le MCPConnectorRegistry, stockée en DB dans `mcp_connector_definitions`.
 */
export interface MCPConnectorDefinition {
  id: string                       // identifiant unique, ex: 'github', 'notion'
  name: string
  description: string
  version: string                  // SemVer
  author?: string
  icon: string                     // URL ou identifiant d'icône (lucide, etc.)
  category: ConnectorCategory
  capabilities: ConnectorCapability[]
  configFields: ConfigField[]      // champs du formulaire de configuration
  tools: MCPTool[]                 // tools MCP exposés au LLM
  resourceTemplates?: MCPResourceTemplate[]
  prompts?: MCPPrompt[]
  docsUrl?: string
  tags?: string[]
}

/**
 * Instance active d'un connecteur dans un workspace.
 * Stockée en DB dans `workspace_mcp_bindings`.
 */
export interface WorkspaceMCPBinding {
  id: string
  workspaceId: string
  connectorId: string
  config: Record<string, unknown>  // valeurs non-secrètes
  secretRefIds: string[]           // références SecretRef pour les credentials
  enabled: boolean
  createdAt: string
  updatedAt: string
}

/**
 * Binding MCP personnel d'un utilisateur (indépendant des workspaces).
 * Stocké en DB dans `user_mcp_bindings`. Monté dans tous les workspaces de l'utilisateur.
 * Distinct de WorkspaceMCPBinding : suit l'utilisateur, pas le workspace.
 */
export interface UserMCPBinding {
  id: string
  userId: string
  connectorId: string
  config: Record<string, unknown>  // valeurs non-secrètes
  secretRefIds: string[]           // références SecretRef — jamais de credential en clair
  enabled: boolean
  createdAt: string
  updatedAt: string
}

/**
 * Message échangé avec le MCP gateway (backend Rust).
 */
export interface MCPCallRequest {
  bindingId: string
  type: 'tool_call' | 'resource_read' | 'prompt_get'
  toolName?: string
  resourceUri?: string
  promptName?: string
  arguments?: Record<string, unknown>
}

export interface MCPCallResponse {
  success: boolean
  result?: unknown
  error?: { code: string; message: string }
}
