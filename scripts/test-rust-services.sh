#!/usr/bin/env sh
set -eu

find_rust_services() {
  find microservices -mindepth 2 -maxdepth 2 -name Cargo.toml -print | sort
}

env_value() {
  key="$1"
  file="$2"

  [ -f "$file" ] || return 0

  sed -n "s/^${key}=//p" "$file" | head -n 1 | tr -d '\r' | sed 's/^"//; s/"$//'
}

db_is_reachable() {
  service_dir="$1"
  db_host="${APP__DATABASE__HOST:-$(env_value "APP__DATABASE__HOST" "${service_dir}/.env.development")}"
  db_port="${APP__DATABASE__PORT:-$(env_value "APP__DATABASE__PORT" "${service_dir}/.env.development")}"
  db_host="${db_host:-localhost}"
  db_port="${db_port:-5433}"

  command -v nc >/dev/null 2>&1 && nc -z -w 2 "$db_host" "$db_port" >/dev/null 2>&1
}

if ! find_rust_services | grep -q .; then
  printf '%s\n' "No Rust services found under microservices/."
  exit 0
fi

find_rust_services | while IFS= read -r manifest; do
  service_dir=$(dirname "$manifest")
  printf '\n%s\n' "Testing Rust service: ${service_dir}"

  if [ -d "${service_dir}/migrations" ]; then
    (
      cd "$service_dir"
      cargo test --locked --lib --all-features
    )

    if db_is_reachable "$service_dir"; then
      printf '%s\n' "Database reachable for ${service_dir}; running DB-backed tests."
      (
        cd "$service_dir"
        cargo test --locked --test controllers --all-features
      )
    else
      printf '%s\n' "Database not reachable for ${service_dir}; skipped DB-backed controller tests."
    fi
  else
    (
      cd "$service_dir"
      cargo test --locked --all-features
    )
  fi
done
