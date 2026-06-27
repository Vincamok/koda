export type {
  MCPConnectorDefinition,
  WorkspaceMCPBinding,
  UserMCPBinding,
  MCPTool,
  MCPResource,
  MCPResourceTemplate,
  MCPPrompt,
  MCPCallRequest,
  MCPCallResponse,
  ConfigField,
  ConnectorCategory,
  ConnectorCapability,
  JsonSchema,
} from './types'

export { mcpRegistry } from './registry'

// Connecteurs built-in
export { githubConnector }   from './connectors/github'
export { notionConnector }   from './connectors/notion'
export { postgresConnector } from './connectors/postgres'
export { httpConnector }     from './connectors/http'
export { slackConnector }    from './connectors/slack'
export { jiraConnector }     from './connectors/jira'

import { mcpRegistry }       from './registry'
import { githubConnector }   from './connectors/github'
import { notionConnector }   from './connectors/notion'
import { postgresConnector } from './connectors/postgres'
import { httpConnector }     from './connectors/http'
import { slackConnector }    from './connectors/slack'
import { jiraConnector }     from './connectors/jira'

// Enregistrement automatique des connecteurs built-in
mcpRegistry.register(githubConnector)
mcpRegistry.register(notionConnector)
mcpRegistry.register(postgresConnector)
mcpRegistry.register(httpConnector)
mcpRegistry.register(slackConnector)
mcpRegistry.register(jiraConnector)
