use serde::{Deserialize, Serialize};

/// État cinématique du véhicule à un instant donné.
///
/// Convention des axes (SAE) :
///   x : vers l'avant du véhicule
///   y : vers la gauche
///   ψ : lacet positif = virage à gauche
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleState {
    // ── Cinématique ────────────────────────────────────────────────────
    /// Vitesse longitudinale dans le repère véhicule (m/s)
    pub vx: f64,
    /// Vitesse latérale dans le repère véhicule (m/s)
    pub vy: f64,
    /// Vitesse de lacet (rad/s)
    pub yaw_rate: f64,

    // ── Accélérations mesurées (pour les résidus) ──────────────────────
    /// Accélération latérale mesurée (g)
    pub acc_lateral_g: f64,
    /// Accélération longitudinale mesurée (g)
    pub acc_longitudinal_g: f64,

    // ── Position (optionnel) ───────────────────────────────────────────
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub heading: Option<f64>,

    // ── Roues (optionnel — modèles 7-DDL et plus) ─────────────────────
    /// Débattement de suspension [FL, FR, RL, RR] (m)
    pub suspension_travel: Option<[f64; 4]>,
    /// Charge verticale sur chaque roue [FL, FR, RL, RR] (N)
    pub tyre_load: Option<[f64; 4]>,
    /// Angle de dérive de chaque pneu [FL, FR, RL, RR] (rad)
    pub tyre_slip_angle: Option<[f64; 4]>,
}

/// Entrées de commande du véhicule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleInput {
    /// Angle de braquage du volant (rad)
    pub steering_wheel_rad: f64,
    /// Gaz normalisé 0..1
    pub throttle: f64,
    /// Frein normalisé 0..1
    pub brake: f64,
    /// Rapport engagé
    pub gear: Option<i8>,
}

impl VehicleState {
    /// Construit un état depuis un sample de télémétrie AC.
    /// `vx` est estimé depuis la vitesse scalaire (hypothèse : glisse latérale faible).
    pub fn from_sample(s: &impl telemetry::TelemetrySample) -> Self {
        let a = s.acceleration_g();
        Self {
            vx: s.speed_ms() as f64,
            vy: 0.0, // non mesurable directement depuis RTCarInfo
            yaw_rate: s.yaw_rate_rads() as f64,
            acc_lateral_g: a[0] as f64,
            acc_longitudinal_g: a[1] as f64,
            x: None,
            y: None,
            heading: None,
            suspension_travel: s.suspension_travel_m()
                .map(|t| t.map(|v| v as f64)),
            tyre_load: s.tyre_load_n()
                .map(|l| l.map(|v| v as f64)),
            tyre_slip_angle: s.tyre_slip_angle_rad()
                .map(|a| a.map(|v| v as f64)),
        }
    }
}

impl VehicleInput {
    pub fn from_sample(
        s: &impl telemetry::TelemetrySample,
        // steering_ratio: f64,
    ) -> Self {
        Self {
            // Le braquage télémétrie est l'angle volant — on divise par le rapport de direction
            // steering_wheel_rad: s.steering_angle_rad() as f64 / steering_ratio,
            steering_wheel_rad: s.steering_angle_rad() as f64,
            throttle: s.throttle_norm() as f64,
            brake: s.brake_norm() as f64,
            gear: s.gear(),
        }
    }
}