> 한국어: [`../ko/ADR/ADR-015-shared-postgres-and-multitenant.md`](../ko/ADR/ADR-015-shared-postgres-and-multitenant.md)

# ADR-015: Shared PostgreSQL + Multi-tenant Isolation on a Single Host

## Status

Accepted (2026-05-10).

## Context

`servers/docker-compose.yml` ran two PostgreSQL instances: `plane-db`
for Plane and `mm-postgres` for Mattermost. Both apps speak the same
RDBMS protocol — the split was historical inertia. The cost was real:

- ~600MB extra RAM (two PG processes)
- Operators run two backups, two upgrades
- `mm-postgres` shipped `postgres:18-alpine`, outside Mattermost
  10.11's officially supported range (13–17)

In addition, the stack was meant to support **several operators
sharing one host under their own user accounts** (internal demos,
training environments). Without explicit isolation:

- Container/volume name collisions when `COMPOSE_PROJECT_NAME` falls
  back to the directory name
- Host port collisions on `PLANE_PORT`, `MM_PORT`, and trial-app's
  hardcoded `3000`
- Shared TLS domains and shared secrets bleed across operators

## Decision

### 1. Consolidate Postgres into one `pg-shared` instance

- Image: `postgres:15.7-alpine` — Plane's officially recommended
  version, also in Mattermost 10.11's supported range.
- Single volume `pg-shared-data`.
- `init/init-databases.sh` runs at first boot to create the second
  role+DB (mattermost). The plane DB is created by Postgres's own
  entrypoint via `POSTGRES_DB`.
- `max_connections=1500` (Plane gunicorn pool + MM pool + headroom).
- The hardening `mm-postgres` had (`security_opt: no-new-privileges`,
  `read_only: true`, `tmpfs`) is preserved on the shared instance.

### 2. Namespace every externally-visible resource per operator

- `COMPOSE_PROJECT_NAME=genasis-${USER}` — enforced by the helper
  script. This single variable namespaces container names, networks,
  **and** volumes automatically.
- Host ports are derived from UID:
  - `PLANE_PORT = 38400 + (uid % 50)`
  - `MM_PORT    = 38500 + (uid % 50)`
  - `TRIAL_APP_PORT = 3100 + (uid % 50)`
- The script probes occupancy with `ss`/`lsof` and walks forward to
  the next free triple if the computed slot is taken.
- All secrets (`*_PASSWORD`, `*_SECRET_KEY`, `*_SHARED_SECRET`) are
  generated with `openssl rand -hex 30`.

### 3. trial-app participates in the same isolation

- `trial-app/docker-compose.yml` switched its `./data` bind mount to
  a named volume `trial-app-data` so `COMPOSE_PROJECT_NAME` actually
  scopes the data dir.
- `trial-app/.env` introduces `COMPOSE_PROJECT_NAME`, `TRIAL_APP_PORT`,
  and `TRIAL_SHARED_SECRET`.
- `setup-user-env.sh` writes both `servers/.env` and `trial-app/.env`
  in one shot, mirroring `TRIAL_SHARED_SECRET` so the Rust trial
  provider routes to the operator's own trial-app.

### 4. Document the per-user Caddy split

A single `import /etc/caddy/sites/genasis-*.caddy` line in
`/etc/caddy/Caddyfile` lets each operator drop a `genasis-${USER}.caddy`
fragment that routes their sub-domains to their host ports. Root just
runs `systemctl reload caddy` after a new file lands.

## Consequences

**Easier**:
- ~4 concurrent operators on a 32GB host (was 2–3 before).
- Backup/upgrade procedures cut roughly in half.
- New operator onboarding = run `setup-user-env.sh` + `docker compose up`.

**Harder**:
- The shared PG is a single point of failure — one process down takes
  both apps with it.
- Not recommended for HA-SLA production. The ADR and migration guide
  call this out and tell operators to keep the split layout in that
  case.
- Existing deployments must `pg_dump` / `pg_restore` to migrate (see
  the migration guide).

**Foreclosed**:
- Independent Postgres major-version upgrades for Plane vs Mattermost.
  After consolidation they share a major version. If one app ever
  requires PG 16 while the other locks to PG 15, the layout has to
  split again.

## Verification

- `docker compose config` validates both compose files (compose v2).
- On a fresh host, `setup-user-env.sh` → `docker compose up -d` →
  Plane API health + MM ping return 200.
- Two operators on the same host produce no conflicts on container
  names, host ports, or volume names (`docker ps`, `ss -tln`).
- trial-app `/api/events/stream` continues to flow SSE updates
  independent of the shared PG.

## References

- Implementation: `servers/docker-compose.yml`,
  `servers/init/init-databases.sh`,
  `servers/scripts/setup-user-env.sh`,
  `trial-app/docker-compose.yml`.
- Migration guide:
  [`../MIGRATE-PG-CONSOLIDATION.md`](../MIGRATE-PG-CONSOLIDATION.md).
- Related ADR: ADR-013 (trial bridge config SSOT) — the `[trial]`
  section ↔ trial-app routing now extends through this ADR's
  `TRIAL_SHARED_SECRET` mirroring.
