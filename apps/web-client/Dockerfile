FROM node:20-alpine AS builder
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_OPTIONS="--max-old-space-size=512"

COPY package.json package-lock.json ./
COPY packages/shared-types/package.json ./packages/shared-types/
COPY packages/i18n/package.json ./packages/i18n/
COPY packages/api-client/package.json ./packages/api-client/
COPY apps/web-client/package.json ./apps/web-client/
COPY packages/shared-types ./packages/shared-types
COPY packages/i18n ./packages/i18n
COPY packages/api-client ./packages/api-client
COPY apps/web-client ./apps/web-client

RUN npm ci && \
    npm run build --workspace=@koda/shared-types 2>/dev/null || true && \
    npm run build --workspace=@koda/i18n 2>/dev/null || true && \
    npm run build --workspace=@koda/api-client 2>/dev/null || true && \
    cd apps/web-client && npx next build && \
    cd /app && rm -rf node_modules

FROM node:20-alpine AS runner
ENV NODE_ENV=production PORT=3003
RUN addgroup --system --gid 1001 nodejs && adduser --system --uid 1001 nextjs
COPY --from=builder /app/apps/web-client/.next/standalone ./
COPY --from=builder /app/apps/web-client/.next/static ./.next/static
USER nextjs
EXPOSE 3003
CMD ["node", "server.js"]
