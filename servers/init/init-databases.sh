#!/usr/bin/env sh
# Bootstrap the shared Postgres instance with one role+database per app.
#
# Runs exactly once on a fresh data dir (the postgres entrypoint executes
# anything in /docker-entrypoint-initdb.d/ on initial cluster init only).
# On restart with an existing pgdata this script is skipped — that is
# what we want, since dropping/recreating roles would clobber app data.
#
# The bootstrap superuser is the Plane role (POSTGRES_USER on the
# pg-shared service). We piggy-back on that to create the Mattermost
# role and DB.

set -eu

: "${PLANE_DB_USER:?PLANE_DB_USER must be set}"
: "${PLANE_DB_PASSWORD:?PLANE_DB_PASSWORD must be set}"
: "${MM_DB_USER:?MM_DB_USER must be set}"
: "${MM_DB_PASSWORD:?MM_DB_PASSWORD must be set}"

# Plane DB is created by the Postgres entrypoint via POSTGRES_DB; we
# only need to add the Mattermost role + DB here.
psql -v ON_ERROR_STOP=1 \
     --username "$PLANE_DB_USER" \
     --dbname  postgres <<-SQL
DO \$\$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '${MM_DB_USER}') THEN
      CREATE ROLE "${MM_DB_USER}" LOGIN PASSWORD '${MM_DB_PASSWORD}';
   END IF;
END
\$\$;

SELECT 'CREATE DATABASE mattermost OWNER "${MM_DB_USER}"'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'mattermost')
\\gexec

GRANT ALL PRIVILEGES ON DATABASE mattermost TO "${MM_DB_USER}";
SQL

echo "init-databases.sh: bootstrapped roles plane, ${MM_DB_USER} and database mattermost"
