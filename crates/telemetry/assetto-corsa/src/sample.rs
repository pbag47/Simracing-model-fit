
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use telemetry_core::TelemetrySample;
use crate::packet::AcRtCarInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcSample {
    pub timestamp_ms: u64,
    pub car_info: AcRtCarInfo,
    /// Rempli depuis RTLap (SUBSCRIBE_SPOT) quand disponible
    pub lap_number: Option<i32>,
    pub lap_time_ms: Option<i32>,
}

impl AcSample {
    pub fn new(car_info: AcRtCarInfo, timestamp_ms: u64) -> Self {
        Self { timestamp_ms, car_info, lap_number: None, lap_time_ms: None }
    }
}

impl TelemetrySample for AcSample {
    fn timestamp_ms(&self) -> u64 { self.timestamp_ms }

    fn speed_ms(&self) -> f32 { self.car_info.speed_ms }

    fn acceleration_g(&self) -> Vector3<f32> {
        // doc AC : horizontal=lateral, frontal=longitudinal, vertical=vertical
        Vector3::new(
            self.car_info.acc_g_horizontal,
            self.car_info.acc_g_frontal,
            self.car_info.acc_g_vertical,
        )
    }

    fn yaw_rate_rads(&self) -> f32 {
        // RTCarInfo n'expose pas directement le lacet — on le calcule à partir
        // de la vitesse et de l'accélération latérale : ψ̇ ≈ ay / v
        // (valable hors vitesses très faibles)
        let v = self.car_info.speed_ms;
        if v > 1.0 {
            self.car_info.acc_g_horizontal * 9.81 / v
        } else {
            0.0
        }
    }

    fn steering_angle_rad(&self) -> f32 {
        // AC fournit le braquage en degrés — conversion en radians
        self.car_info.steer.to_radians()
    }
    fn throttle_norm(&self) -> f32 { self.car_info.gas.clamp(0.0, 1.0) }
    fn brake_norm(&self) -> f32 { self.car_info.brake.clamp(0.0, 1.0) }
    fn gear(&self) -> Option<i8> { Some(self.car_info.gear as i8) }

    fn wheel_speed_ms(&self) -> [f32; 4] {
        // tyre_radius est fourni dynamiquement par AC — on l'utilise directement
        let ws = self.car_info.wheel_angular_speed;
        let r  = self.car_info.tyre_loaded_radius; // rayon sous charge
        std::array::from_fn(|i| ws[i] * r[i])
    }

    fn suspension_travel_m(&self) -> Option<[f32; 4]> {
        Some(self.car_info.suspension_height)
    }

    fn tyre_slip_angle_rad(&self) -> Option<[f32; 4]> {
        Some(self.car_info.slip_angle)
    }

    fn tyre_slip_ratio(&self) -> Option<[f32; 4]> {
        Some(self.car_info.slip_ratio)
    }

    fn tyre_load_n(&self) -> Option<[f32; 4]> {
        Some(self.car_info.load)
    }

    fn position_m(&self) -> Option<Vector3<f32>> {
        let c = self.car_info.car_coordinates;
        Some(Vector3::new(c[0], c[1], c[2]))
    }

    fn lap_distance_m(&self) -> Option<f32> {
        // car_position_normalized est 0..1 sur la longueur du circuit
        // On ne connaît pas la longueur ici — on expose normalized tel quel
        // et on laissera le post-traitement calculer la distance réelle
        None
    }

    fn simulator_id(&self) -> &'static str { "assetto_corsa" }
}

