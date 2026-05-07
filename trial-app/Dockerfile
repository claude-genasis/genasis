# syntax=docker/dockerfile:1.7
ARG NODE_IMAGE=node:20-alpine

# ─────────────────────────────────────────── deps ──
FROM ${NODE_IMAGE} AS deps
WORKDIR /app
RUN apk add --no-cache python3 make g++ libc6-compat
COPY package.json package-lock.json ./
RUN npm ci

# ─────────────────────────────────────────── build ──
FROM ${NODE_IMAGE} AS builder
WORKDIR /app
RUN apk add --no-cache python3 make g++ libc6-compat
ENV NEXT_TELEMETRY_DISABLED=1
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

# ─────────────────────────────────────────── run ──
FROM ${NODE_IMAGE} AS runner
WORKDIR /app
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    PORT=3000 \
    HOSTNAME=0.0.0.0
RUN apk add --no-cache libc6-compat \
 && addgroup -g 1001 -S nodejs \
 && adduser  -u 1001 -S nextjs -G nodejs \
 && mkdir -p /data \
 && chown -R nextjs:nodejs /data
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static    ./.next/static
COPY --from=builder --chown=nextjs:nodejs /app/public          ./public
USER nextjs
EXPOSE 3000
VOLUME ["/data"]
ENV DATABASE_PATH=/data/trial.db
CMD ["node", "server.js"]
