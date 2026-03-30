use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Version du format de fichier — à incrémenter si le format change
pub const FORMAT_VERSION: u32 = 1;

/// Magic bytes pour identifier les fichiers .srf
pub const MAGIC: &[u8; 4] = b"SRF\x01";

/// Métadonnées de session — stockées en JSON en tête de fichier
/// pour être lisibles sans outil spécial (ex: `head -c 512 session.srf`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub format_version: u32,
    pub simulator: String,
    pub car: Option<String>,
    pub track: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub sample_count: usize,
    pub duration_s: f64,
    pub sample_rate_hz: Option<f64>,
    /// Quels canaux optionnels sont présents dans cette session
    pub channels: ChannelManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelManifest {
    pub suspension_travel: bool,
    pub tyre_slip: bool,
    pub tyre_load: bool,
    pub tyre_temp: bool,
    pub tyre_pressure: bool,
    pub position: bool,
    pub lap_distance: bool,
    pub gear: bool,
}

/// Session complète prête à être sérialisée
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredSession<S> {
    pub metadata: SessionMetadata,
    pub samples: Vec<S>,
}

impl<S: serde::Serialize> StoredSession<S> {
    pub fn channel_manifest_from_samples(_samples: &[S]) -> ChannelManifest {
        // La détection fine des canaux se fait au niveau du simulateur spécifique.
        // Par défaut on retourne un manifest vide — les crates telemetry-* peuvent
        // surcharger via SessionStore::save_ac() etc.
        ChannelManifest::default()
    }
}