use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use chrono::Utc;
use serde::{de::DeserializeOwned, Serialize};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use tracing::{info, debug};

use telemetry_core::{Session, TelemetrySample};
use crate::format::{
    ChannelManifest, FORMAT_VERSION, MAGIC, SessionMetadata, StoredSession,
};
use crate::error::StoreError;

pub struct SessionStore;

impl SessionStore {
    /// Sauvegarde une session dans un fichier `.srf`.
    ///
    /// Format binaire :
    ///   [4 bytes magic] [4 bytes longueur header] [N bytes JSON header] [payload gzip+bincode]
    pub fn save<S, P>(
        session: &Session<S>,
        path: P,
        channels: ChannelManifest,
    ) -> Result<(), StoreError>
    where
        S: TelemetrySample + Serialize,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Sauvegarde session → {}", path.display());

        let metadata = SessionMetadata {
            format_version: FORMAT_VERSION,
            simulator: session.simulator.to_string(),
            car: session.car_name.clone(),
            track: session.track_name.clone(),
            recorded_at: Utc::now(),
            sample_count: session.samples.len(),
            duration_s: session.duration_s(),
            sample_rate_hz: session.sample_rate_hz(),
            channels,
        };

        // Sérialisation du payload en bincode puis compression gzip
        let payload_raw = bincode::serialize(&session.samples)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let mut compressed = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::fast());
            encoder.write_all(&payload_raw)?;
            encoder.finish()?;
        }

        // Header JSON
        let header_json = serde_json::to_vec_pretty(&metadata)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        // Écriture du fichier
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(MAGIC)?;
        // Longueur du header sur 4 octets little-endian
        writer.write_all(&(header_json.len() as u32).to_le_bytes())?;
        writer.write_all(&header_json)?;
        writer.write_all(&compressed)?;
        writer.flush()?;

        let file_size_kb = std::fs::metadata(path)?.len() / 1024;
        info!(
            "Session sauvegardée : {} samples, {:.1}s, {} Ko → {}",
            session.samples.len(),
            session.duration_s(),
            file_size_kb,
            path.display()
        );

        Ok(())
    }

    /// Charge une session depuis un fichier `.srf`.
    pub fn load<S, P>(path: P) -> Result<(SessionMetadata, Vec<S>), StoreError>
    where
        S: DeserializeOwned,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Chargement session ← {}", path.display());

        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);

        // Vérification magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(StoreError::InvalidFile(
                "Magic bytes incorrects — pas un fichier .srf".into(),
            ));
        }

        // Lecture longueur header
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let header_len = u32::from_le_bytes(len_bytes) as usize;

        // Lecture et parse du header JSON
        let mut header_bytes = vec![0u8; header_len];
        reader.read_exact(&mut header_bytes)?;
        let metadata: SessionMetadata = serde_json::from_slice(&header_bytes)
            .map_err(|e| StoreError::InvalidFile(format!("Header JSON invalide : {e}")))?;

        // Vérification version
        if metadata.format_version != FORMAT_VERSION {
            return Err(StoreError::IncompatibleVersion {
                expected: FORMAT_VERSION,
                found: metadata.format_version,
            });
        }

        // Décompression + désérialisation payload
        let mut compressed = Vec::new();
        reader.read_to_end(&mut compressed)?;

        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut raw = Vec::new();
        decoder.read_to_end(&mut raw)?;

        let samples: Vec<S> = bincode::deserialize(&raw)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        info!(
            "Session chargée : {} samples, {:.1}s, sim={}",
            samples.len(),
            metadata.duration_s,
            metadata.simulator
        );

        Ok((metadata, samples))
    }

    /// Lit uniquement les métadonnées d'un fichier sans charger les samples.
    /// Utile pour afficher un résumé rapide sans tout désérialiser.
    pub fn read_metadata<P: AsRef<Path>>(path: P) -> Result<SessionMetadata, StoreError> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(StoreError::InvalidFile("Magic bytes incorrects".into()));
        }

        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let header_len = u32::from_le_bytes(len_bytes) as usize;

        let mut header_bytes = vec![0u8; header_len];
        reader.read_exact(&mut header_bytes)?;

        serde_json::from_slice(&header_bytes)
            .map_err(|e| StoreError::InvalidFile(format!("Header JSON invalide : {e}")))
    }
}