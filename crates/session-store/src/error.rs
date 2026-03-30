use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Erreur I/O : {0}")]
    Io(#[from] std::io::Error),

    #[error("Erreur de sérialisation : {0}")]
    Serialization(String),

    #[error("Fichier de session invalide ou corrompu : {0}")]
    InvalidFile(String),

    #[error("Version de format incompatible : attendu {expected}, trouvé {found}")]
    IncompatibleVersion { expected: u32, found: u32 },
}