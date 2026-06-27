# Koda

Plateforme de workspaces développeur à la demande — containers Docker isolés, IDE web intégré, pipelines CI/CD et connecteurs MCP.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Internet                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
               ┌───────▼────────┐
               │  sozu (proxy)  │  HTTP + TCP routing
               └───────┬────────┘
          ┌────────────┼────────────┐
          │            │            │
   ┌──────▼──────┐ ┌───▼────┐ ┌────▼──────┐
   │  Dashboard  │ │  API   │ │ Web IDE   │
   │  Next.js    │ │  Axum  │ │ Next.js   │
   └─────────────┘ └───┬────┘ └───────────┘
                       │
          ┌────────────┼──────────────┐
          │            │              │
   ┌──────▼──────┐ ┌───▼──────┐ ┌────▼──────────┐
   │ Orchestrator│ │  Worker  │ │  MCP Gateway  │
   │  (bollard)  │ │(pipelines│ │  (connecteurs)│
   └──────┬──────┘ │ GC cron) │ └───────────────┘
          │        └──────────┘
   ┌──────▼──────────────────────┐
   │  Docker (via socket proxy)  │
   │  Workspace containers       │
   └─────────────────────────────┘
          │
   ┌──────▼──────────────────────┐
   │  PostgreSQL  │  Redis       │
   └─────────────────────────────┘
```

## Services

| Service | Langage | Rôle |
|---------|---------|------|
| `apps/api` | Rust / Axum | API REST + sessions + auth |
| `apps/dashboard` | Next.js 14 | Interface utilisateur principale |
| `apps/admin` | Next.js 14 | Panel super-admin |
| `apps/web-client` | Next.js 14 | IDE web (Monaco + chat IA) |
| `services/orchestrator` | Rust | Cycle de vie containers Docker (bollard) |
| `services/worker` | Rust | Pipelines CI/CD, GC volumes, cron |
| `services/gateway` | Rust | Client sozu (routes HTTP + TCP) |
| `services/mcp-gateway` | Rust | Connecteurs MCP (Redis Streams consumer) |
| `services/git-manager` | Rust | Opérations Git asynchrones |

## Packages partagés

| Package | Rôle |
|---------|------|
| `packages/shared-types` | Types TypeScript communs |
| `packages/api-client` | Client HTTP typé (généré depuis OpenAPI) |
| `packages/themes` | ThemeRegistry + 4 skins |
| `packages/mcp-connectors` | Registre TypeScript connecteurs MCP |
| `packages/i18n` | Clés i18n + helpers (FR/EN/ES/DE) |

---

## Prérequis

- Rust 1.79+ (`rustup`, via `rust-toolchain.toml`)
- Node.js 20+ + npm 10+
- Docker + docker-compose
- PostgreSQL 16
- Redis 7

## Démarrage (développement)

```bash
# Infra
docker compose -f infra/docker/docker-compose.yml up -d

# Variables d'environnement
cp apps/api/.env.example apps/api/.env
# Éditer DATABASE_URL, REDIS_URL, SESSION_SECRET, etc.

# Migrations
cd apps/api && sqlx migrate run --source ../../infra/migrations

# API
cargo run -p koda-api

# Dashboard
npm install
npm run dev --workspace=apps/dashboard
```

## Documentation API

Swagger UI disponible sur `http://localhost:8080/swagger-ui` quand l'API tourne.

Spec OpenAPI : `http://localhost:8080/api-docs/openapi.json`

## Tests E2E

```bash
cd e2e
cp ../.env.example .env  # configurer TEST_USER_EMAIL + TEST_USER_PASSWORD
npx playwright test
```

---

## Versioning

SemVer — voir [CHANGELOG.md](./CHANGELOG.md) et [ROADMAP.md](./ROADMAP.md).

Version courante : **0.4.0** (phases 0–3 complètes, phase 4 MVP en cours).

## Sécurité

Les invariants de sécurité sont documentés dans [AGENTS.md](./AGENTS.md). Points clés :
- Aucun secret en clair — `SecretRef` uniquement
- Docker socket via proxy uniquement (`docker-socket-proxy`)
- Resource limits obligatoires sur tous les containers
- RLS PostgreSQL activé sur 13 tables critiques
- Rate limiting par IP (300/min) et par utilisateur (600/min)
