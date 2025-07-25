#!/usr/bin/env bash
set -x
set -eo pipefail

# sprawdzenie czy sqlx-cli jest już zainstalowane
if ! [ -x "$(command -v sqlx)" ]; then
echo >&2 "Error: sqlx is not installed."
echo >&2 "Use:"
echo >&2 " cargo install --version='~0.8' sqlx-cli \
--no-default-features --features rustls,postgres"
echo >&2 "to install it."
exit 1
fi

# Check if a custom parameter has been set, otherwise use default values
DB_PORT="${POSTGRES_PORT:=5432}"
SUPERUSER="${SUPERUSER:=postgres}"
SUPERUSER_PWD="${SUPERUSER_PWD:=password}"
APP_USER="${APP_USER:=app}"
APP_USER_PWD="${APP_USER_PWD:=secret}"
APP_DB_NAME="${APP_DB_NAME:=newsletter}"
    # Launch postgres using Docker
CONTAINER_NAME="postgres"
CZY_PG_ADMIN=true

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
    then


    docker run \
    --env POSTGRES_USER=${SUPERUSER} \
    --env POSTGRES_PASSWORD=${SUPERUSER_PWD} \
    --health-cmd="pg_isready -U ${SUPERUSER} || exit 1" \
    --health-interval=1s \
    --health-timeout=5s \
    --health-retries=5 \
    --publish "${DB_PORT}":5432 \
    --detach \
    --name "${CONTAINER_NAME}" \
    postgres -N 1000
    # ^ Increased maximum number of connections for testing purposes
    # Wait for Postgres to be ready to accept connections
    until [ \
    "$(docker inspect -f "{{.State.Health.Status}}" ${CONTAINER_NAME})" == \
    "healthy" \
    ]; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
    done
    >&2 echo "Postgres is up and running on port ${DB_PORT}!"

fi

# Sprawdzenie czy user istnieje
USER_EXISTS=$(docker exec $CONTAINER_NAME psql -U $SUPERUSER -tAc "SELECT 1 FROM pg_roles WHERE rolname='$APP_USER'")

if [ "$USER_EXISTS" = "" ]; then
    #Create the application user
    CREATE_QUERY="CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';"
    docker exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${CREATE_QUERY}"
    # Grant create db privileges to the app user
    GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
    docker exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${GRANT_QUERY}"
fi

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"


DATABASE_URL=postgres://${APP_USER}:${APP_USER_PWD}@localhost:${DB_PORT}/${APP_DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run
>&2 echo "Postgres has been migrated, ready to go!"

#Uruchomienie pgadmin

if [[  "${CZY_PG_ADMIN}" ]]
    then
docker run -p 80:80 \
    -e 'PGADMIN_DEFAULT_EMAIL=pszota@interia.pl' \
    -e 'PGADMIN_DEFAULT_PASSWORD=kukuruku' \
    -v pgadmin-data:/var/lib/pgadmin \
    -d dpage/pgadmin4
fi

