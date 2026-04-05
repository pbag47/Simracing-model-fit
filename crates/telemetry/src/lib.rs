use anyhow::Error;
// Ré-exports publics du core — tout le monde peut les utiliser
pub use telemetry_core::sample::TelemetrySample;
pub use telemetry_core::session::Session;
pub use telemetry_core::error::TelemetryError;

// Types internes — on les garde accessible pour session-store et viewer
// qui en ont besoin pour la désérialisation, mais via telemetry uniquement
pub use telemetry_assetto_corsa::AcSample;
pub use telemetry_assetto_corsa::AcUdpReader;
pub use telemetry_assetto_corsa::reader::AcSessionInfo;

// ── Type unifié ──────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// Sample générique indépendant du simulateur.
/// Permet à session-store, viewer et identification de ne pas
/// dépendre des crates simulateur spécifiques.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericSample {
    Ac(AcSample),
    // Lmu(LmuSample),  // décommenté quand LMU sera implémenté
}

impl TelemetrySample for GenericSample {
    fn timestamp_ms(&self) -> u64 {
        match self { Self::Ac(s) => s.timestamp_ms() }
    }
    fn speed_ms(&self) -> f32 {
        match self { Self::Ac(s) => s.speed_ms() }
    }
    fn acceleration_g(&self) -> nalgebra::Vector3<f32> {
        match self { Self::Ac(s) => s.acceleration_g() }
    }
    fn yaw_rate_rads(&self) -> f32 {
        match self { Self::Ac(s) => s.yaw_rate_rads() }
    }
    fn steering_angle_rad(&self) -> f32 {
        match self { Self::Ac(s) => s.steering_angle_rad() }
    }
    fn throttle_norm(&self) -> f32 {
        match self { Self::Ac(s) => s.throttle_norm() }
    }
    fn brake_norm(&self) -> f32 {
        match self { Self::Ac(s) => s.brake_norm() }
    }
    fn gear(&self) -> Option<i8> {
        match self { Self::Ac(s) => s.gear() }
    }
    fn wheel_speed_ms(&self) -> [f32; 4] {
        match self { Self::Ac(s) => s.wheel_speed_ms() }
    }
    fn suspension_travel_m(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.suspension_travel_m() }
    }
    fn tyre_slip_angle_rad(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.tyre_slip_angle_rad() }
    }
    fn tyre_slip_ratio(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.tyre_slip_ratio() }
    }
    fn tyre_load_n(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.tyre_load_n() }
    }
    fn tyre_temp_celsius(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.tyre_temp_celsius() }
    }
    fn tyre_pressure_kpa(&self) -> Option<[f32; 4]> {
        match self { Self::Ac(s) => s.tyre_pressure_kpa() }
    }
    fn position_m(&self) -> Option<nalgebra::Vector3<f32>> {
        match self { Self::Ac(s) => s.position_m() }
    }
    fn lap_distance_m(&self) -> Option<f32> {
        match self { Self::Ac(s) => s.lap_distance_m() }
    }
    fn simulator_id(&self) -> &'static str {
        match self { Self::Ac(s) => s.simulator_id() }
    }
}

// ── Canaux supplémentaires spécifiques à un simulateur ───────────────────

/// Canaux qui n'existent pas dans le trait TelemetrySample mais sont
/// utiles pour le viewer ou l'analyse — exposés ici sans dépendance aux crates internes.
pub struct ExtraChannels {
    pub engine_rpm: Option<f32>,
}

impl GenericSample {
    pub fn extra(&self) -> ExtraChannels {
        match self {
            Self::Ac(s) => ExtraChannels {
                engine_rpm: Some(s.car_info.engine_rpm),
            },
        }
    }
}


/// Convertit une Session<AcSample> en Session<GenericSample>.
/// La conversion est zero-copy pour les métadonnées, et wrap chaque sample
/// dans GenericSample::Ac.
fn session_ac_to_generic(session: Session<AcSample>) -> Session<GenericSample> {
    Session {
        samples: session.samples.into_iter().map(GenericSample::Ac).collect(),
        car_name: session.car_name,
        track_name: session.track_name,
        simulator: session.simulator,
    }
}

// ── API haut niveau — ce que la CLI utilise ───────────────────────────────

/// Identifiant de simulateur, passé en argument de ligne de commande.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SimulatorKind {
    #[value(name = "ac")]
    AssettoCorsa,
    // #[value(name = "lmu")]
    // LeMansUltimate,
}

pub async fn record(
    simulator: SimulatorKind, 
    local_addr: &str, 
    udp_server_addr: &str, 
    max_samples: usize,
    ) -> Result<Session<GenericSample>, Error> {
    match simulator {
        SimulatorKind::AssettoCorsa{} => {
            let (reader, session_info) = AcUdpReader::connect(
                &local_addr, &udp_server_addr).await?;

            println!("Connecté :");
            println!("  Voiture  : {}", session_info.car_name);
            println!("  Pilote   : {}", session_info.driver_name);
            println!("  Circuit  : {}/{}", session_info.track_name, session_info.track_config);

            let mut session = reader.record_session(max_samples).await?;
            session.car_name   = Some(session_info.car_name);
            session.track_name = Some(session_info.track_name);

            reader.dismiss().await?;
            Ok(session_ac_to_generic(session))
        }
    }
}