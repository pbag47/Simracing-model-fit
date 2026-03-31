
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::timeout;
use bytemuck::bytes_of;
use tracing::{debug, info, warn};

use telemetry_core::{Session, TelemetryError};
use crate::packet::{
    AcHandshakePacket, AcHandshakeResponse, AcRtCarInfo, AcRtLap,
};
use crate::sample::AcSample;

/// Informations de session récupérées lors du handshake
#[derive(Debug, Clone)]
pub struct AcSessionInfo {
    pub car_name:    String,
    pub driver_name: String,
    pub track_name:  String,
    pub track_config: String,
}

pub struct AcUdpReader {
    socket: UdpSocket,
    ac_addr: SocketAddr,
}

impl AcUdpReader {
    /// Crée le reader et établit le handshake complet avec AC.
    ///
    /// `local_addr`  : adresse locale d'écoute, ex. "0.0.0.0:9996"
    /// `ac_addr`     : adresse d'AC, ex. "127.0.0.1:9996"
    ///                 (identique si AC tourne en local)
    pub async fn connect(
        local_addr: &str,
        ac_addr: &str,
    ) -> Result<(Self, AcSessionInfo), TelemetryError> {
        let socket = UdpSocket::bind(local_addr).await?;
        let ac_addr: SocketAddr = ac_addr.parse()
            .map_err(|e| TelemetryError::InvalidPacket(format!("Adresse invalide : {e}")))?;

        info!("Socket UDP lié sur {local_addr}, AC cible : {ac_addr}");

        // ── Étape 1 : HANDSHAKE ──────────────────────────────────────────
        let pkt = AcHandshakePacket::handshake();
        socket.send_to(bytes_of(&pkt), ac_addr).await?;
        debug!("Handshake envoyé (operationId=0)");

        // ── Étape 2 : attendre la réponse AC (408 octets) ────────────────
        let mut buf = [0u8; 1024];
        let session_info = loop {
            let recv = timeout(Duration::from_secs(10), socket.recv_from(&mut buf)).await
                .map_err(|_| TelemetryError::InvalidPacket(
                    "Timeout : pas de réponse AC en 10s — le jeu est-il lancé ?".into()
                ))??;

            let (len, from) = recv;
            debug!("Paquet reçu : {len} octets depuis {from}");


            if len >= AcHandshakeResponse::SIZE {
                if let Some(response) = AcHandshakeResponse::from_bytes(&buf[..len]) {
                    let info = AcSessionInfo {
                        car_name:     response.car_name,
                        driver_name:  response.driver_name,
                        track_name:   response.track_name,
                        track_config: response.track_config,
                    };
                    info!(
                        "Handshake réponse : car={}, driver={}, track={}/{}",
                        info.car_name, info.driver_name, info.track_name, info.track_config
                    );
                    break info;
                }
            } else {
                debug!("Paquet de {len} octets ignoré (trop petit pour handshake response)");
            }
        };

        // ── Étape 3 : SUBSCRIBE_UPDATE ───────────────────────────────────
        let pkt = AcHandshakePacket::subscribe_update();
        socket.send_to(bytes_of(&pkt), ac_addr).await?;
        info!("Subscribe envoyé (operationId=1) — streaming démarré");

        Ok((Self { socket, ac_addr }, session_info))
    }

    /// Enregistre une session jusqu'à `max_samples` samples RTCarInfo.
    /// Gère aussi les RTLap (fin de tour) reçus en parallèle.
    pub async fn record_session(
        &self,
        max_samples: usize,
    ) -> Result<Session<AcSample>, TelemetryError> {
        let mut session = Session::new("assetto_corsa");
        let mut buf = [0u8; 1024];

        let car_info_size = std::mem::size_of::<AcRtCarInfo>();

        while session.samples.len() < max_samples {
            let (len, _) = self.socket.recv_from(&mut buf).await?;

            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            if len >= car_info_size && buf[0] == b'a' {
                // RTCarInfo — identifier == 'a'
                let mut aligned = vec![0u8; car_info_size];
                aligned.copy_from_slice(&buf[..car_info_size]);
                let car_info: AcRtCarInfo = *bytemuck::from_bytes(&aligned);
                let sample = AcSample::new(car_info, ts);
                debug!(
                    speed_kmh = car_info.speed_kmh,
                    lap = car_info.lap_count,
                    "RTCarInfo #{}", session.samples.len()
                );
                session.push(sample);
            } else if len >= AcRtLap::SIZE {
                // RTLap — spot event fin de tour
                if let Some(lap) = AcRtLap::from_bytes(&buf[..len]) {
                    info!(
                        "Tour {} terminé — {} — temps : {:?}",
                        lap.lap,
                        lap.driver_name,
                        lap.lap_time_duration()
                    );
                    if let Some(last) = session.samples.last_mut() {
                        last.lap_number  = Some(lap.lap);
                        last.lap_time_ms = Some(lap.time_ms);
                    }
                }
            } else if len == 12 {
                // Probablement un handshaker retourné par AC — ignoré
                debug!("Paquet 12 octets ignoré (handshaker echo ?)");
            } else {
                warn!("Paquet de taille inattendue : {len} octets");
            }
        }

        info!(
            "Session : {} samples, {:.1}s, {:.0} Hz",
            session.samples.len(),
            session.duration_s(),
            session.sample_rate_hz().unwrap_or(0.0)
        );
        Ok(session)
    }

    /// Envoie un DISMISS propre à AC avant de fermer.
    pub async fn dismiss(&self) -> Result<(), TelemetryError> {
        let pkt = AcHandshakePacket::dismiss();
        self.socket.send_to(bytes_of(&pkt), self.ac_addr).await?;
        info!("DISMISS envoyé — connexion fermée proprement");
        Ok(())
    }
}






// // use std::net::SocketAddr;
// use std::time::{SystemTime, UNIX_EPOCH};
// use tokio::net::UdpSocket;
// use tracing::{
//     // debug, 
//     warn,
//     info
// };

// use telemetry_core::{Session, TelemetryError};
// use crate::packet::AcPhysicsPacket;
// use crate::sample::AcSample;

// pub struct AcUdpReader {
//     socket: UdpSocket,
// }

// impl AcUdpReader {
//     /// Ouvre le socket UDP sur l'adresse fournie.
//     /// Exemple : "0.0.0.0:9996"
//     pub async fn bind(addr: &str) -> Result<Self, TelemetryError> {
//         let socket = UdpSocket::bind(addr).await?;
//         tracing::info!("AcUdpReader en écoute sur {}", addr);
//         Ok(Self { socket })
//     }

//     /// Enregistre une session jusqu'à `max_samples` samples.
//     pub async fn record_session(
//         &self,
//         max_samples: usize,
//     ) -> Result<Session<AcSample>, TelemetryError> {
//         let mut session = Session::new("assetto_corsa");
//         let mut buf = [0u8; 2048];
//         let mut sample_counter = 0;

//         // while session.samples.len() < max_samples {
//          while sample_counter < max_samples {
//             let (len, _addr) = self.socket.recv_from(&mut buf).await?;
//             match self.parse_packet(&buf[..len]) {
//                 Ok(Some(sample)) => {
//                     println!(
//                         "sample #{}", session.samples.len()
//                     );
//                     session.push(sample);
//                 }
//                 Ok(None) => {println!("unknown packet") /* paquet Graphics ou inconnu, on ignore */ }
//                 Err(e) => warn!("Paquet invalide : {e}"),
//             }
//             sample_counter = sample_counter + 1
//         }

//         info!(
//             "Session enregistrée : {} samples, {:.1}s",
//             session.samples.len(),
//             session.duration_s()
//         );
//         Ok(session)
//     }

//     fn parse_packet(
//         &self,
//         data: &[u8],
//     ) -> Result<Option<AcSample>, TelemetryError> {
//         let physics_size = std::mem::size_of::<AcPhysicsPacket>();

//         // Expected packet sizes:
//         // 328 -> Car info, physics
//         // 212 -> Lap info
//         // 408 -> Handshake response with session info

//         println!("Received #{} bytes, expected #{}", data.len(), physics_size); 

//         if data.len() < physics_size {
//             // Peut être un paquet Graphics (plus petit) — pas une erreur
//             return Ok(None);
//         }

//         // Copie dans un buffer aligné avant le cast
//         let mut aligned = [0u8; 328];
//         aligned[..physics_size].copy_from_slice(&data[..physics_size]);

//         // Safety: AcPhysicsPacket est Pod (bytemuck), pas d'UB possible
//         let packet: AcPhysicsPacket = *bytemuck::from_bytes(&aligned);

//         let ts = SystemTime::now()
//             .duration_since(UNIX_EPOCH)
//             .unwrap_or_default()
//             .as_millis() as u64;

//         Ok(Some(AcSample::new(packet, ts)))
//     }
// }