# Build
cargo build

# Écouter AC en UDP (port 9996 par défaut dans AC)
RUST_LOG=debug cargo run --bin simracing-fit -- record-ac

# Ou sur un port custom
cargo run --bin simracing-fit -- record-ac --addr 0.0.0.0:9997 --max-samples 500