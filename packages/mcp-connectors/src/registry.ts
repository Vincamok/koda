import type { MCPConnectorDefinition, ConnectorCategory, ConnectorCapability } from './types'

type ChangeListener = (connectors: MCPConnectorDefinition[]) => void

/**
 * Registre global des connecteurs MCP.
 * Pattern identique au ThemeRegistry — même extensibilité, même cycle de vie.
 *
 * Usage (intégration d'un connecteur custom) :
 *   mcpRegistry.register(myCustomConnector)
 *
 * Usage (chargement depuis marketplace) :
 *   await mcpRegistry.loadFromUrl('https://marketplace.koda.dev/connectors.json')
 */
class MCPConnectorRegistry {
  private connectors = new Map<string, MCPConnectorDefinition>()
  private listeners  = new Set<ChangeListener>()

  register(connector: MCPConnectorDefinition): void {
    this.connectors.set(connector.id, connector)
    this.notify()
  }

  unregister(id: string): void {
    this.connectors.delete(id)
    this.notify()
  }

  get(id: string): MCPConnectorDefinition | undefined {
    return this.connectors.get(id)
  }

  list(): MCPConnectorDefinition[] {
    return [...this.connectors.values()]
  }

  listByCategory(category: ConnectorCategory): MCPConnectorDefinition[] {
    return this.list().filter((c) => c.category === category)
  }

  listByCapability(capability: ConnectorCapability): MCPConnectorDefinition[] {
    return this.list().filter((c) => c.capabilities.includes(capability))
  }

  search(query: string): MCPConnectorDefinition[] {
    const q = query.toLowerCase()
    return this.list().filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.description.toLowerCase().includes(q) ||
        c.tags?.some((t) => t.toLowerCase().includes(q))
    )
  }

  has(id: string): boolean {
    return this.connectors.has(id)
  }

  async loadFromUrl(url: string): Promise<MCPConnectorDefinition[]> {
    const res = await fetch(url)
    if (!res.ok) throw new Error(`Échec chargement connecteurs depuis ${url}: ${res.status}`)
    const defs: MCPConnectorDefinition[] = await res.json()
    defs.forEach((d) => this.register(d))
    return defs
  }

  onChange(listener: ChangeListener): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  private notify(): void {
    const connectors = this.list()
    this.listeners.forEach((l) => l(connectors))
  }
}

export const mcpRegistry = new MCPConnectorRegistry()
