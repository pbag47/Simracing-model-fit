use nalgebra::Vector3;

/// Un instant de télémétrie normalisé, indépendant du simulateur source.
/// Les unités sont toujours : m/s, m/s², rad, rad/s, N, N·m, °C
pub trait TelemetrySample: Send + Sync + 'static {
    // --- Temporel ---
    fn timestamp_ms(&self) -> u64;

    // --- Cinématique véhicule ---
    fn speed_ms(&self) -> f32;
    fn velocity_world(&self) -> Option<Vector3<f32>> { None }
    fn acceleration_g(&self) -> Vector3<f32>; // lateral, longitudinal, vertical
    fn yaw_rate_rads(&self) -> f32;
    fn pitch_rate_rads(&self) -> Option<f32> { None }
    fn roll_rate_rads(&self) -> Option<f32> { None }
    fn engine_rpm(&self) -> Option<f32> { None }

    // --- Direction & commandes ---
    fn steering_angle_rad(&self) -> f32;
    fn throttle_norm(&self) -> f32;   // 0.0 .. 1.0
    fn brake_norm(&self) -> f32;      // 0.0 .. 1.0
    fn gear(&self) -> Option<i8> { None }

    // --- Roues (ordre : FL, FR, RL, RR) ---
    fn wheel_speed_ms(&self) -> [f32; 4];
    fn suspension_travel_m(&self) -> Option<[f32; 4]> { None }
    fn tyre_slip_angle_rad(&self) -> Option<[f32; 4]> { None }
    fn tyre_slip_ratio(&self) -> Option<[f32; 4]> { None }
    fn tyre_load_n(&self) -> Option<[f32; 4]> { None }
    fn tyre_temp_celsius(&self) -> Option<[f32; 4]> { None }
    fn tyre_pressure_kpa(&self) -> Option<[f32; 4]> { None }

    // --- Position (optionnel selon sim) ---
    fn position_m(&self) -> Option<Vector3<f32>> { None }
    fn lap_distance_m(&self) -> Option<f32> { None }

    // --- Métadonnées ---
    fn simulator_id(&self) -> &'static str;
}

/// Résumé des canaux disponibles pour un sample donné.
/// Utile pour savoir quels modèles sont identifiables avec cette source.
#[derive(Debug, Clone)]
pub struct ChannelAvailability {
    pub suspension_travel: bool,
    pub tyre_slip: bool,
    pub tyre_load: bool,
    pub tyre_temp: bool,
    pub position: bool,
    pub lap_distance: bool,
}

impl ChannelAvailability {
    pub fn from_sample(s: &impl TelemetrySample) -> Self {
        // On crée un sample factice juste pour tester les Options
        // (on ne peut pas appeler sur `s` sans données — on laisse le sim
        //  déclarer ses capacités via une méthode séparée sur le reader)
        let _ = s;
        Self {
            suspension_travel: false,
            tyre_slip: false,
            tyre_load: false,
            tyre_temp: false,
            position: false,
            lap_distance: false,
        }
    }
}