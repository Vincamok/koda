FROM node:20-alpine AS base
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED=1

FROM base AS deps
COPY package.json package-lock.json ./
COPY packages/shared-types/package.json ./packages/shared-types/
COPY packages/i18n/package.json ./packages/i18n/
COPY packages/api-client/package.json ./packages/api-client/
COPY apps/dashboard/package.json ./apps/dashboard/
RUN npm ci

FROM base AS builder
ENV NODE_OPTIONS="--max-old-space-size=512"
COPY --from=deps /app/node_modules ./node_modules
COPY packages/shared-types ./packages/shared-types
COPY packages/i18n ./packages/i18n
COPY packages/api-client ./packages/api-client
COPY apps/dashboard ./apps/dashboard
RUN npm run build --workspace=@koda/shared-types 2>/dev/null || true
RUN npm run build --workspace=@koda/i18n 2>/dev/null || true
RUN npm run build --workspace=@koda/api-client 2>/dev/null || true
WORKDIR /app/apps/dashboard
RUN npx next build

FROM base AS runner
ENV NODE_ENV=production PORT=3001
RUN addgroup --system --gid 1001 nodejs && adduser --system --uid 1001 nextjs
COPY --from=builder /app/apps/dashboard/.next/standalone ./
COPY --from=builder /app/apps/dashboard/.next/static ./.next/static
COPY --from=builder /app/apps/dashboard/public ./public 2>/dev/null || true
USER nextjs
EXPOSE 3001
CMD ["node", "server.js"]
