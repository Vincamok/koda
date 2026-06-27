/**
 * API error codes → i18n key mapping.
 * The API returns English machine codes; the frontend translates via these keys.
 */
export const API_ERROR_KEYS: Record<string, string> = {
  NOT_FOUND: 'errors.not_found',
  UNAUTHORIZED: 'errors.unauthorized',
  FORBIDDEN: 'errors.forbidden',
  CONFLICT: 'errors.conflict',
  BAD_REQUEST: 'errors.bad_request',
  QUOTA_EXCEEDED: 'errors.quota_exceeded',
  VALIDATION_ERROR: 'errors.validation',
  INTERNAL_ERROR: 'errors.internal',
  DATABASE_ERROR: 'errors.internal',
  WORKSPACE_NOT_FOUND: 'errors.workspace.not_found',
  WORKSPACE_QUOTA_EXCEEDED: 'errors.workspace.quota_exceeded',
  PLUGIN_NOT_FOUND: 'errors.plugin.not_found',
  GIT_CLONE_FAILED: 'errors.git.clone_failed',
  MCP_CONNECTOR_ERROR: 'errors.mcp.connector_error',
}

export function apiErrorKey(code: string): string {
  return API_ERROR_KEYS[code] ?? 'errors.unknown'
}
