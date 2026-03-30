# Build
cargo build

# Écouter AC en UDP (port 9996 par défaut dans AC)
RUST_LOG=debug cargo run --bin simracing-fit -- record-ac

# Ou sur un port custom
cargo run --bin simracing-fit -- record-ac --addr 0.0.0.0:9997 --max-samples 500

# Enregistrer une session AC
cargo run --bin simracing-fit -- record-ac --max-samples 1000 --output ma_session.srf

# Inspecter le fichier sans charger les données
cargo run --bin simracing-fit -- info ma_session.srf

# Rejouer et afficher les 20 premiers samples
cargo run --bin simracing-fit -- replay ma_session.srf --samples 20
