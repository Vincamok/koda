import type { MCPConnectorDefinition } from '../types'

export const notionConnector: MCPConnectorDefinition = {
  id:          'notion',
  name:        'Notion',
  description: 'Lecture et écriture dans des pages, bases de données et blocs Notion.',
  version:     '1.0.0',
  icon:        'notebook',
  category:    'project',
  capabilities: ['read-context', 'execute-tools', 'search'],
  docsUrl:     'https://developers.notion.com',
  tags:        ['docs', 'wiki', 'knowledge', 'database'],

  configFields: [
    {
      key:         'token',
      label:       'Integration Token',
      type:        'password',
      required:    true,
      secret:      true,
      description: 'Token d\'intégration interne Notion (starts with secret_)',
    },
    {
      key:         'root_page_id',
      label:       'Page racine (optionnel)',
      type:        'text',
      required:    false,
      placeholder: 'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx',
      description: 'Limite l\'accès à une sous-arborescence',
    },
  ],

  tools: [
    {
      name:        'notion_search',
      description: 'Recherche dans les pages et bases de données Notion.',
      inputSchema: {
        type: 'object',
        required: ['query'],
        properties: {
          query:  { type: 'string' },
          filter: { type: 'string', enum: ['page', 'database'], description: 'Filtre par type' },
          limit:  { type: 'number', description: 'Défaut: 10' },
        },
      },
    },
    {
      name:        'notion_get_page',
      description: 'Retourne le contenu d\'une page Notion.',
      inputSchema: {
        type: 'object',
        required: ['page_id'],
        properties: {
          page_id: { type: 'string' },
        },
      },
    },
    {
      name:        'notion_create_page',
      description: 'Crée une nouvelle page dans une base de données ou page parente.',
      inputSchema: {
        type: 'object',
        required: ['parent_id', 'title'],
        properties: {
          parent_id: { type: 'string' },
          title:     { type: 'string' },
          content:   { type: 'string', description: 'Markdown converti en blocs Notion' },
        },
      },
    },
    {
      name:        'notion_append_block',
      description: 'Ajoute du contenu à la fin d\'une page.',
      inputSchema: {
        type: 'object',
        required: ['page_id', 'content'],
        properties: {
          page_id: { type: 'string' },
          content: { type: 'string', description: 'Markdown' },
        },
      },
    },
  ],

  resourceTemplates: [
    {
      uriTemplate: 'notion://{page_id}',
      name:        'Page Notion',
      mimeType:    'text/markdown',
    },
  ],
}
