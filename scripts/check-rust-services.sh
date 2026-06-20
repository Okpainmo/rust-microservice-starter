#!/usr/bin/env sh
set -eu

find_rust_services() {
  find microservices -mindepth 2 -maxdepth 2 -name Cargo.toml -print | sort
}

if ! find_rust_services | grep -q .; then
  printf '%s\n' "No Rust services found under microservices/."
  exit 0
fi

find_rust_services | while IFS= read -r manifest; do
  service_dir=$(dirname "$manifest")
  printf '\n%s\n' "Checking Rust service: ${service_dir}"

  (
    cd "$service_dir"
    cargo check --locked --all-targets --all-features
    cargo fmt --all -- --check
    cargo clippy --locked --all-targets --all-features -- -D warnings
  )
done
