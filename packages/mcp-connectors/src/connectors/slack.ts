import type { MCPConnectorDefinition } from '../types'

export const slackConnector: MCPConnectorDefinition = {
  id:          'slack',
  name:        'Slack',
  description: 'Lecture de canaux, envoi de messages et recherche dans Slack.',
  version:     '1.0.0',
  icon:        'message-square',
  category:    'communication',
  capabilities: ['read-context', 'execute-tools', 'search'],
  docsUrl:     'https://api.slack.com',
  tags:        ['chat', 'team', 'notifications', 'messaging'],

  configFields: [
    {
      key:      'bot_token',
      label:    'Bot Token',
      type:     'password',
      required: true,
      secret:   true,
      description: 'Token OAuth du bot Slack (xoxb-...)',
    },
    {
      key:         'default_channel',
      label:       'Canal par défaut',
      type:        'text',
      required:    false,
      placeholder: '#général',
    },
  ],

  tools: [
    {
      name:        'slack_post_message',
      description: 'Envoie un message dans un canal ou en DM.',
      inputSchema: {
        type: 'object',
        required: ['channel', 'text'],
        properties: {
          channel: { type: 'string', description: 'ID ou nom du canal (#channel ou @user)' },
          text:    { type: 'string' },
          thread_ts: { type: 'string', description: 'Pour répondre dans un fil' },
        },
      },
    },
    {
      name:        'slack_search_messages',
      description: 'Recherche dans les messages Slack.',
      inputSchema: {
        type: 'object',
        required: ['query'],
        properties: {
          query:   { type: 'string' },
          channel: { type: 'string', description: 'Limite la recherche à un canal' },
          limit:   { type: 'number', description: 'Défaut: 20' },
        },
      },
    },
    {
      name:        'slack_list_channels',
      description: 'Liste les canaux publics de l\'espace de travail.',
      inputSchema: {
        type: 'object',
        properties: {
          limit: { type: 'number', description: 'Défaut: 50' },
        },
      },
    },
  ],
}
