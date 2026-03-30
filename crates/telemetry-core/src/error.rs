use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Erreur réseau : {0}")]
    Network(#[from] std::io::Error),

    #[error("Paquet trop court : attendu {expected} octets, reçu {received}")]
    PacketTooShort { expected: usize, received: usize },

    #[error("Version de protocole inconnue : {0}")]
    UnknownProtocolVersion(u32),

    #[error("Paquet invalide : {0}")]
    InvalidPacket(String),
}