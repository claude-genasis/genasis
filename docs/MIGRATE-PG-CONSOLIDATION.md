> 한국어: [`./ko/MIGRATE-PG-CONSOLIDATION.md`](./ko/MIGRATE-PG-CONSOLIDATION.md)

# PostgreSQL Consolidation — Plane + Mattermost → Single PG

Per ADR-015, `servers/docker-compose.yml` now consolidates the per-app
`plane-db` and `mm-postgres` instances into one `pg-shared`
(postgres:15.7-alpine). This document covers migrating an **existing
deployment** to the new layout. Fresh deployments need only
`docker compose up -d` and can ignore this file.

## Summary of changes

| Item | Before | After |
|---|---|---|
| Plane Postgres | `plane-db` service, `postgres:15.7-alpine` | single `pg-shared`, `postgres:15.7-alpine` |
| MM Postgres | `mm-postgres` service, `postgres:18-alpine` | single `pg-shared`, `postgres:15.7-alpine` |
| Plane DB volume | `plane-pgdata` | `pg-shared-data` |
| MM DB volume | `mm-pgdata` | `pg-shared-data` (shared) |
| Plane `DATABASE_URL` | `…@plane-db/plane` | `…@pg-shared/plane` |
| MM datasource | `…@mm-postgres:5432/mattermost…` | `…@pg-shared:5432/mattermost…` |
| `max_connections` | Plane 1000 / MM default | unified 1500 |
| trial-app data dir | `./data` bind mount | named volume `trial-app-data` |

## Pre-flight

- Check out the git revision with the new compose (do **not** `up` yet)
- Verify the existing stack is healthy (`docker compose ps` all healthy)
- Postgres dump may briefly lock — schedule a maintenance window
- Free disk space ≥ 1.5× the combined DB size

## Migration steps

### 1. Dump both databases

```bash
cd servers/

# Run inside the existing containers so passwords stay off the host shell.
docker compose exec -T plane-db \
  pg_dump -U "$PLANE_POSTGRES_USER" -d plane -Fc -Z9 \
  > /tmp/plane-$(date +%Y%m%d).dump

docker compose exec -T mm-postgres \
  pg_dump -U "$MM_POSTGRES_USER" -d mattermost -Fc -Z9 \
  > /tmp/mattermost-$(date +%Y%m%d).dump

ls -lh /tmp/{plane,mattermost}-*.dump
```

> ⚠️ Plane's worker / beat may be mid-write. For a perfectly clean
> dump pause them first:
> ```bash
> docker compose stop plane-worker plane-beat
> # ↑ run dump
> docker compose start plane-worker plane-beat
> ```

### 2. Stop the existing stack (preserve volumes)

```bash
docker compose down
# Never use `down -v` — that wipes the volumes and the dump becomes
# your only copy.
```

### 3. Pull the new compose

`git pull` to get the new `docker-compose.yml` plus
`init/init-databases.sh`. Make sure `servers/.env` has:

```env
COMPOSE_PROJECT_NAME=genasis-...
PLANE_POSTGRES_USER=plane
PLANE_POSTGRES_PASSWORD=<same as before>
MM_POSTGRES_USER=mmuser
MM_POSTGRES_PASSWORD=<same as before>
```

> Passwords must match the role records baked into the dumps. Generating
> new passwords here will break GRANTs at restore time, so keep the old
> ones (rotate later if you must, after restore is verified).

### 4. Bring up `pg-shared` only

```bash
docker compose up -d pg-shared

# Wait for the init script to create the mattermost role + DB (~5s).
docker compose logs -f pg-shared | grep -m1 "init-databases.sh"

# Verify both databases exist.
docker compose exec pg-shared \
  psql -U plane -d postgres -c '\l'
# Expect to see both `plane` and `mattermost`.
```

### 5. Restore both dumps

```bash
# Plane
docker compose exec -T pg-shared \
  pg_restore -U plane -d plane --clean --if-exists \
  < /tmp/plane-$(date +%Y%m%d).dump

# Mattermost
docker compose exec -T pg-shared \
  pg_restore -U mmuser -d mattermost --clean --if-exists \
  < /tmp/mattermost-$(date +%Y%m%d).dump
```

> Plane dumps may emit warnings about GRANTs referencing roles other than
> `plane`. Data is restored correctly — only critical errors warrant
> attention.

### 6. Bring up the rest + health check

```bash
docker compose up -d

# Plane
curl -fsSL "http://localhost:${PLANE_PORT}/api/v1/health/" | jq .
# Mattermost
curl -fsSL "http://localhost:${MM_PORT}/api/v4/system/ping" | jq .

# The migrators run idempotently once and exit clean.
docker compose logs plane-migrator | tail -20
docker compose logs mattermost     | tail -20
```

### 7. Migrate trial-app data (if applicable)

Old `trial-app/docker-compose.yml` used `./data` as a bind mount. The
new file uses a named volume `trial-app-data`.

```bash
cd ../trial-app
# Bring it up once to materialise the empty volume, then stop.
docker compose up -d
docker compose stop

# Copy the host ./data sqlite into the new named volume.
docker run --rm \
  -v "$(pwd)/data:/from:ro" \
  -v "${COMPOSE_PROJECT_NAME}_trial-app-data:/to" \
  alpine sh -c "cp -av /from/. /to/"

docker compose up -d
```

### 8. Cleanup after burn-in

- Run the new stack for ≥24h and exercise issue creation, MM messaging,
  and migrator re-runs to confirm health.
- Remove the legacy volumes once you trust the new stack:
  ```bash
  docker volume rm "${COMPOSE_PROJECT_NAME}_plane-pgdata"
  docker volume rm "${COMPOSE_PROJECT_NAME}_mm-pgdata"
  ```
- Move or delete `/tmp/*.dump`.

## Rollback

If anything misbehaves, take the new stack down and revert the compose:

```bash
docker compose down
git checkout <previous-commit> -- servers/docker-compose.yml
docker compose up -d
```

As long as you haven't reached step 8, the legacy `plane-pgdata` /
`mm-pgdata` volumes are picked back up automatically — zero data loss.

## Operational changes after consolidation

| Task | Before | After |
|---|---|---|
| PG backup | two `pg_dump` runs | one `pg_dumpall` or two DB-scoped dumps |
| PG upgrade | independent schedules | single shared schedule for both apps |
| Disk monitoring | two volumes | one volume `pg-shared-data` |
| Failure isolation | one PG down ≠ other app down | both impacted. **HA-critical setups should not consolidate** |

## FAQ

**Q. PG 18 → PG 15 is a downgrade for Mattermost — won't data break?**
A. Mattermost 10.11 officially supports Postgres 13–17. PG 18 was the
unofficial one. Dump format is forward/backward compatible across major
versions, so restore works.

**Q. Is `max_connections=1500` excessive?**
A. Plane gunicorn workers + worker + beat + live + redis pool ≈ 800–1000;
MM default pool ≈ 200–400. 1500 leaves headroom. The connection-limit
itself costs little RAM (`shared_buffers` is the real driver).

**Q. Both apps go down if the shared PG dies — isn't that risky?**
A. Yes. **HA-SLA production setups should keep the split layout.** The
consolidation is for trial / demo / dev environments where the RAM and
ops simplicity wins. Re-read ADR-015 § Trade-offs before adopting in
prod.

## References

- ADR-015 — Postgres consolidation decision (forthcoming)
- [`servers/README.md`](../servers/README.md) — full guide for the new layout
- [`docs/ADR/ADR-013-trial-bridge-config-wiring.md`](ADR/ADR-013-trial-bridge-config-wiring.md) — trial routing SSOT
