# Build
cargo build

# Enregistrer une session AC (local)
cargo run --bin simracing-fit -- record --simulator ac --local-addr 0.0.0.0:9997 --udp-server-addr 127.0.0.1:9996 --max-samples 100 --output session.srf

# Inspecter un fichier de session
cargo run --bin simracing-fit -- info session.srf

# Rejouer les 20 premiers samples
cargo run --bin simracing-fit -- replay ma_session.srf --samples 20

# Visualisation graphique d'une session
cargo run --bin viewer -- session.srf