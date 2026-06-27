import type { MCPConnectorDefinition } from '../types'

export const postgresConnector: MCPConnectorDefinition = {
  id:          'postgres',
  name:        'PostgreSQL',
  description: 'Interrogation et exploration de bases de données PostgreSQL.',
  version:     '1.0.0',
  icon:        'database',
  category:    'database',
  capabilities: ['read-context', 'execute-tools'],
  tags:        ['database', 'sql', 'postgres'],

  configFields: [
    {
      key:         'connection_string',
      label:       'URL de connexion',
      type:        'password',
      required:    true,
      secret:      true,
      placeholder: 'postgresql://user:pass@host:5432/dbname',
    },
    {
      key:         'readonly',
      label:       'Lecture seule',
      type:        'boolean',
      required:    false,
      defaultValue: true,
      description: 'Recommandé : interdit les requêtes INSERT/UPDATE/DELETE',
    },
    {
      key:         'max_rows',
      label:       'Nombre max de lignes par requête',
      type:        'number',
      required:    false,
      defaultValue: 100,
    },
  ],

  tools: [
    {
      name:        'postgres_query',
      description: 'Exécute une requête SQL SELECT et retourne les résultats.',
      inputSchema: {
        type: 'object',
        required: ['sql'],
        properties: {
          sql:    { type: 'string', description: 'Requête SQL (SELECT uniquement si readonly=true)' },
          params: { type: 'array',  items: { type: 'string' }, description: 'Paramètres liés ($1, $2...)' },
        },
      },
    },
    {
      name:        'postgres_list_tables',
      description: 'Liste les tables et vues du schéma courant.',
      inputSchema: {
        type: 'object',
        properties: {
          schema: { type: 'string', description: 'Schéma cible (défaut: public)' },
        },
      },
    },
    {
      name:        'postgres_describe_table',
      description: 'Retourne le schéma détaillé d\'une table (colonnes, types, contraintes).',
      inputSchema: {
        type: 'object',
        required: ['table'],
        properties: {
          table:  { type: 'string' },
          schema: { type: 'string', description: 'Défaut: public' },
        },
      },
    },
  ],
}
