#!/usr/bin/env bash
# Runs the Laravel reference (the `main` branch) on http://127.0.0.1:8315
# for recording golden fixtures (scripts/laravel-reference/record.py).
#
#   scripts/laravel-reference/run.sh setup        export main, install, build assets
#   scripts/laravel-reference/run.sh start        fresh database, serve
#   scripts/laravel-reference/run.sh artisan ...  run artisan in the reference container
#   scripts/laravel-reference/run.sh php '...'    run PHP code with the app booted (tinker)
#   scripts/laravel-reference/run.sh sql "..."    run SQL against the reference database
#   scripts/laravel-reference/run.sh stop
#
# The checkout lives in $LARAVEL_REF_DIR (default /tmp/bluebook-laravel-ref),
# never inside this repository. MySQL 8 like production (case-insensitive
# collation), the database queue and cache like production, the scope
# catalogue seeded (not the SDE), and ReferenceServiceProvider.php
# answering every outgoing HTTP call and signing in test users.
# Containers are named bluebook-laravel-*, on the bluebook-laravel network.
set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)
dir=${LARAVEL_REF_DIR:-/tmp/bluebook-laravel-ref}
image=bluebook-laravel-ref
network=bluebook-laravel
port=8315

run() {
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -e COMPOSER_HOME=/tmp/composer \
    --network "$network" -v "$dir/app:/app" "$@"
}

mysql_up() {
  docker network inspect "$network" >/dev/null 2>&1 || docker network create "$network" >/dev/null
  if ! docker inspect bluebook-laravel-mysql >/dev/null 2>&1; then
    docker run -d --name bluebook-laravel-mysql --network "$network" \
      -e MYSQL_ROOT_PASSWORD=bluebook -e MYSQL_DATABASE=standings \
      mysql:8.4 --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci >/dev/null
  fi
  until docker exec bluebook-laravel-mysql mysql -uroot -pbluebook -e 'select 1' >/dev/null 2>&1; do sleep 1; done
}

case "${1:-}" in
  setup)
    docker build -t "$image" "$repo/scripts/laravel-reference"
    rm -rf "$dir" && mkdir -p "$dir/app"
    git -C "$repo" archive main | tar -x -C "$dir/app"
    cp "$repo/scripts/laravel-reference/env" "$dir/app/.env"
    cp "$repo/scripts/laravel-reference/ReferenceServiceProvider.php" "$dir/app/app/Providers/"
    sed -i 's/^    AppServiceProvider::class,/&\n    App\\Providers\\ReferenceServiceProvider::class,/' "$dir/app/bootstrap/providers.php"
    mysql_up
    run --name bluebook-laravel-setup "$image" sh -c '
      composer install --no-interaction --no-progress
      npm ci --no-audit --no-fund
      php artisan wayfinder:generate --with-form || true
      npx vite build'
    ;;
  start)
    mysql_up
    rm -f "$dir/app/storage/logs/laravel.log" "$dir/app/storage/logs/http.jsonl"
    docker exec bluebook-laravel-mysql mysql -uroot -pbluebook \
      -e 'drop database if exists standings; create database standings' 2>/dev/null
    run --name bluebook-laravel-migrate "$image" sh -c \
      'php artisan migrate --force >/dev/null && php artisan db:seed --class=EsiScopeSeeder --force >/dev/null'
    docker rm -f bluebook-laravel-ref >/dev/null 2>&1 || true
    docker run -d --name bluebook-laravel-ref -u "$(id -u):$(id -g)" -e HOME=/tmp \
      -e PHP_CLI_SERVER_WORKERS=4 --network "$network" \
      -v "$dir/app:/app" \
      -p "127.0.0.1:$port:$port" "$image" \
      php artisan serve --host=0.0.0.0 --port="$port" --no-reload >/dev/null
    until curl -fs "http://127.0.0.1:$port/up" >/dev/null; do sleep 0.5; done
    echo "Laravel reference on http://127.0.0.1:$port"
    ;;
  artisan)
    shift
    docker exec bluebook-laravel-ref php artisan "$@"
    ;;
  php)
    shift
    docker exec -i bluebook-laravel-ref php artisan tinker --execute="$*"
    ;;
  sql)
    shift
    docker exec -i bluebook-laravel-mysql mysql -uroot -pbluebook --default-character-set=utf8mb4 -N -B -r standings -e "$*" 2>/dev/null
    ;;
  stop)
    docker rm -f bluebook-laravel-ref bluebook-laravel-mysql >/dev/null 2>&1 || true
    docker network rm "$network" >/dev/null 2>&1 || true
    ;;
  *)
    sed -n '2,17p' "$0"
    exit 1
    ;;
esac
