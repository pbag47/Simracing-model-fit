# Build
cargo build

# Enregistrer une session AC (local)
RUST_LOG=debug cargo run --bin simracing-fit -- record-ac \
    --local-addr 0.0.0.0:9997 \
    --ac-addr 127.0.0.1:9996 \
    --max-samples 100 \
    --output session.srf

# Inspecter un fichier de session
cargo run --bin simracing-fit -- info session.srf

# Rejouer les 20 premiers samples
cargo run --bin simracing-fit -- replay ma_session.srf --samples 20

