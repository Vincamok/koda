import type { MCPConnectorDefinition } from '../types'

export const githubConnector: MCPConnectorDefinition = {
  id:          'github',
  name:        'GitHub',
  description: 'Accès aux issues, PRs, commits, code search et actions GitHub.',
  version:     '1.0.0',
  icon:        'github',
  category:    'vcs',
  capabilities: ['read-context', 'execute-tools', 'search'],
  docsUrl:     'https://docs.github.com/rest',
  tags:        ['git', 'code', 'issues', 'pr'],

  configFields: [
    {
      key:         'owner',
      label:       'Organisation / utilisateur',
      type:        'text',
      required:    true,
      placeholder: 'mon-org',
    },
    {
      key:         'repo',
      label:       'Dépôt (optionnel — laissez vide pour tous)',
      type:        'text',
      required:    false,
      placeholder: 'mon-repo',
    },
    {
      key:         'token',
      label:       'Personal Access Token',
      type:        'password',
      required:    true,
      secret:      true,
      description: 'Scopes requis : repo, read:org',
    },
  ],

  tools: [
    {
      name:        'github_list_issues',
      description: 'Liste les issues ouvertes du dépôt.',
      inputSchema: {
        type: 'object',
        properties: {
          state:  { type: 'string', enum: ['open', 'closed', 'all'], description: 'Filtre par état' },
          labels: { type: 'string', description: 'Labels séparés par virgule' },
          limit:  { type: 'number', description: 'Nombre max de résultats (défaut: 20)' },
        },
      },
    },
    {
      name:        'github_get_pr',
      description: 'Retourne le détail d\'une Pull Request.',
      inputSchema: {
        type: 'object',
        required: ['number'],
        properties: {
          number: { type: 'number', description: 'Numéro de la PR' },
        },
      },
    },
    {
      name:        'github_search_code',
      description: 'Recherche dans le code source du dépôt.',
      inputSchema: {
        type: 'object',
        required: ['query'],
        properties: {
          query: { type: 'string', description: 'Requête de recherche GitHub' },
          limit: { type: 'number', description: 'Nombre max de résultats (défaut: 10)' },
        },
      },
    },
    {
      name:        'github_create_issue',
      description: 'Crée une nouvelle issue.',
      inputSchema: {
        type: 'object',
        required: ['title'],
        properties: {
          title:  { type: 'string' },
          body:   { type: 'string' },
          labels: { type: 'array', items: { type: 'string' } },
        },
      },
    },
    {
      name:        'github_comment_pr',
      description: 'Ajoute un commentaire sur une PR.',
      inputSchema: {
        type: 'object',
        required: ['number', 'body'],
        properties: {
          number: { type: 'number' },
          body:   { type: 'string' },
        },
      },
    },
  ],

  resourceTemplates: [
    {
      uriTemplate: 'github://{owner}/{repo}/issues/{number}',
      name:        'Issue GitHub',
      mimeType:    'application/json',
    },
    {
      uriTemplate: 'github://{owner}/{repo}/pulls/{number}',
      name:        'Pull Request GitHub',
      mimeType:    'application/json',
    },
    {
      uriTemplate: 'github://{owner}/{repo}/blob/{branch}/{path}',
      name:        'Fichier source',
      mimeType:    'text/plain',
    },
  ],
}
