import type { MCPConnectorDefinition } from '../types'

export const jiraConnector: MCPConnectorDefinition = {
  id:          'jira',
  name:        'Jira',
  description: 'Consultation et mise à jour de tickets, sprints et projets Jira.',
  version:     '1.0.0',
  icon:        'trello',
  category:    'project',
  capabilities: ['read-context', 'execute-tools', 'search'],
  docsUrl:     'https://developer.atlassian.com/cloud/jira/platform/rest/v3',
  tags:        ['tickets', 'sprint', 'agile', 'atlassian'],

  configFields: [
    {
      key:         'base_url',
      label:       'URL Jira',
      type:        'url',
      required:    true,
      placeholder: 'https://monequipe.atlassian.net',
    },
    {
      key:      'email',
      label:    'Email du compte',
      type:     'text',
      required: true,
    },
    {
      key:      'api_token',
      label:    'API Token',
      type:     'password',
      required: true,
      secret:   true,
      description: 'Généré sur https://id.atlassian.com/manage-profile/security/api-tokens',
    },
    {
      key:         'project_key',
      label:       'Clé projet par défaut (optionnel)',
      type:        'text',
      required:    false,
      placeholder: 'KODA',
    },
  ],

  tools: [
    {
      name:        'jira_search_issues',
      description: 'Recherche des tickets avec une requête JQL.',
      inputSchema: {
        type: 'object',
        required: ['jql'],
        properties: {
          jql:   { type: 'string', description: 'Ex: project = KODA AND status = "In Progress"' },
          limit: { type: 'number', description: 'Défaut: 20' },
        },
      },
    },
    {
      name:        'jira_get_issue',
      description: 'Retourne le détail complet d\'un ticket.',
      inputSchema: {
        type: 'object',
        required: ['issue_key'],
        properties: {
          issue_key: { type: 'string', description: 'Ex: KODA-42' },
        },
      },
    },
    {
      name:        'jira_create_issue',
      description: 'Crée un nouveau ticket.',
      inputSchema: {
        type: 'object',
        required: ['project_key', 'summary', 'issue_type'],
        properties: {
          project_key: { type: 'string' },
          summary:     { type: 'string' },
          description: { type: 'string' },
          issue_type:  { type: 'string', enum: ['Bug', 'Story', 'Task', 'Epic'] },
          priority:    { type: 'string', enum: ['Highest', 'High', 'Medium', 'Low', 'Lowest'] },
          assignee:    { type: 'string', description: 'Account ID Jira' },
        },
      },
    },
    {
      name:        'jira_transition_issue',
      description: 'Change le statut d\'un ticket (ex: To Do → In Progress).',
      inputSchema: {
        type: 'object',
        required: ['issue_key', 'transition_name'],
        properties: {
          issue_key:       { type: 'string' },
          transition_name: { type: 'string', description: 'Nom de la transition (ex: "In Progress")' },
        },
      },
    },
  ],
}
