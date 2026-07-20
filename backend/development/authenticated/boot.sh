#!/bin/sh
set -eu

readonly secret_path=/run/secrets/storage_pgbouncer_admin_password
readonly userlist_path=/etc/hermes/auth/users.txt

umask 077
mkdir -p /etc/hermes/auth
chmod 700 /etc/hermes/auth
password=$(cat "$secret_path")
printf '"hermes_pgbouncer_admin" "%s"\n' "$password" > "$userlist_path"
unset password

exec /entrypoint.sh "$@"
