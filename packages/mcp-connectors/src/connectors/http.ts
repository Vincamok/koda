import type { MCPConnectorDefinition } from '../types'

// Connecteur générique HTTP — permet de brancher n'importe quelle API REST
// sans avoir à écrire un connecteur dédié.
export const httpConnector: MCPConnectorDefinition = {
  id:          'http',
  name:        'API HTTP générique',
  description: 'Connecteur universel pour interroger n\'importe quelle API REST.',
  version:     '1.0.0',
  icon:        'globe',
  category:    'http',
  capabilities: ['read-context', 'execute-tools'],
  tags:        ['rest', 'api', 'http', 'generic', 'custom'],

  configFields: [
    {
      key:         'base_url',
      label:       'URL de base',
      type:        'url',
      required:    true,
      placeholder: 'https://api.exemple.com/v1',
    },
    {
      key:         'auth_type',
      label:       'Type d\'authentification',
      type:        'select',
      required:    true,
      defaultValue: 'none',
      options: [
        { label: 'Aucune',        value: 'none' },
        { label: 'Bearer Token',  value: 'bearer' },
        { label: 'API Key header',value: 'apikey-header' },
        { label: 'Basic Auth',    value: 'basic' },
      ],
    },
    {
      key:         'auth_value',
      label:       'Valeur d\'authentification',
      type:        'password',
      required:    false,
      secret:      true,
      description: 'Token, clé API ou mot de passe selon le type d\'auth',
    },
    {
      key:         'auth_header',
      label:       'Nom du header (si API Key)',
      type:        'text',
      required:    false,
      placeholder: 'X-Api-Key',
    },
    {
      key:         'default_headers',
      label:       'Headers supplémentaires (JSON)',
      type:        'textarea',
      required:    false,
      placeholder: '{"Content-Type": "application/json"}',
    },
  ],

  tools: [
    {
      name:        'http_get',
      description: 'Effectue une requête GET vers un endpoint de l\'API configurée.',
      inputSchema: {
        type: 'object',
        required: ['path'],
        properties: {
          path:   { type: 'string', description: 'Chemin relatif à la base URL, ex: /users/42' },
          params: { type: 'object', description: 'Query params' },
        },
      },
    },
    {
      name:        'http_post',
      description: 'Effectue une requête POST.',
      inputSchema: {
        type: 'object',
        required: ['path'],
        properties: {
          path: { type: 'string' },
          body: { type: 'object', description: 'Corps de la requête (JSON)' },
        },
      },
    },
    {
      name:        'http_patch',
      description: 'Effectue une requête PATCH.',
      inputSchema: {
        type: 'object',
        required: ['path'],
        properties: {
          path: { type: 'string' },
          body: { type: 'object' },
        },
      },
    },
    {
      name:        'http_delete',
      description: 'Effectue une requête DELETE.',
      inputSchema: {
        type: 'object',
        required: ['path'],
        properties: {
          path: { type: 'string' },
        },
      },
    },
  ],
}
