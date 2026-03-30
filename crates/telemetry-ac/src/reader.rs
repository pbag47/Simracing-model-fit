
// use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tracing::{
    // debug, 
    warn,
    info
};

use telemetry_core::{Session, TelemetryError};
use crate::packet::AcPhysicsPacket;
use crate::sample::AcSample;

pub struct AcUdpReader {
    socket: UdpSocket,
}

impl AcUdpReader {
    /// Ouvre le socket UDP sur l'adresse fournie.
    /// Exemple : "0.0.0.0:9996"
    pub async fn bind(addr: &str) -> Result<Self, TelemetryError> {
        let socket = UdpSocket::bind(addr).await?;
        tracing::info!("AcUdpReader en écoute sur {}", addr);
        Ok(Self { socket })
    }

    /// Enregistre une session jusqu'à `max_samples` samples.
    pub async fn record_session(
        &self,
        max_samples: usize,
    ) -> Result<Session<AcSample>, TelemetryError> {
        let mut session = Session::new("assetto_corsa");
        let mut buf = [0u8; 2048];
        let mut sample_counter = 0;

        // while session.samples.len() < max_samples {
         while sample_counter < max_samples {
            let (len, _addr) = self.socket.recv_from(&mut buf).await?;
            match self.parse_packet(&buf[..len]) {
                Ok(Some(sample)) => {
                    println!(
                        "sample #{}", session.samples.len()
                    );
                    session.push(sample);
                }
                Ok(None) => {println!("unknown packet") /* paquet Graphics ou inconnu, on ignore */ }
                Err(e) => warn!("Paquet invalide : {e}"),
            }
            sample_counter = sample_counter + 1
        }

        info!(
            "Session enregistrée : {} samples, {:.1}s",
            session.samples.len(),
            session.duration_s()
        );
        Ok(session)
    }

    fn parse_packet(
        &self,
        data: &[u8],
    ) -> Result<Option<AcSample>, TelemetryError> {
        let physics_size = std::mem::size_of::<AcPhysicsPacket>();

        // Expected packet sizes:
        // 328 -> Car info, physics
        // 212 -> Lap info
        // 408 -> Handshake response with session info

        println!("Received #{} bytes, expected #{}", data.len(), physics_size); 

        if data.len() < physics_size {
            // Peut être un paquet Graphics (plus petit) — pas une erreur
            return Ok(None);
        }

        // Copie dans un buffer aligné avant le cast
        let mut aligned = [0u8; 328];
        aligned[..physics_size].copy_from_slice(&data[..physics_size]);

        // Safety: AcPhysicsPacket est Pod (bytemuck), pas d'UB possible
        let packet: AcPhysicsPacket = *bytemuck::from_bytes(&aligned);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(Some(AcSample::new(packet, ts)))
    }
}