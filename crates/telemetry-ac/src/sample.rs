
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

    fn steering_angle_rad(&self) -> f32 { self.car_info.steer }
    fn throttle_norm(&self) -> f32 { self.car_info.gas.clamp(0.0, 1.0) }
    fn brake_norm(&self) -> f32 { self.car_info.brake.clamp(0.0, 1.0) }
    fn gear(&self) -> Option<i8> { Some(self.car_info.gear as i8) }

    fn wheel_speed_ms(&self) -> [f32; 4] {
        // tyre_radius est fourni dynamiquement par AC — on l'utilise directement
        let ws = &self.car_info.wheel_angular_speed;
        let r  = &self.car_info.tyre_loaded_radius; // rayon sous charge
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






// use nalgebra::Vector3;
// use serde::{Deserialize, Serialize};

// use telemetry_core::TelemetrySample;
// use crate::packet::AcPhysicsPacket;


// /// Sample normalisé construit depuis un AcPhysicsPacket
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AcSample {
//     pub timestamp_ms: u64,
//     pub physics: AcPhysicsPacket,
//     pub lap_distance: Option<f32>,
// }

// impl AcSample {
//     pub fn new(physics: AcPhysicsPacket, timestamp_ms: u64) -> Self {
//         Self { timestamp_ms, physics, lap_distance: None }
//     }
// }

// impl TelemetrySample for AcSample {
//     fn timestamp_ms(&self) -> u64 {
//         self.timestamp_ms
//     }

//     fn speed_ms(&self) -> f32 {
//         self.physics.speed_kmh / 3.6
//     }

//     fn velocity_world(&self) -> Option<Vector3<f32>> {
//         let v = self.physics.velocity;
//         Some(Vector3::new(v[0], v[1], v[2]))
//     }

//     fn acceleration_g(&self) -> Vector3<f32> {
//         let a = self.physics.acc_g;
//         // AC : acc_g[0]=lateral, [1]=longitudinal, [2]=vertical
//         Vector3::new(a[0], a[1], a[2])
//     }

//     fn yaw_rate_rads(&self) -> f32 {
//         // local_angular_vel : [pitch, yaw, roll] en rad/s
//         self.physics.local_angular_vel[1]
//     }

//     fn pitch_rate_rads(&self) -> Option<f32> {
//         Some(self.physics.local_angular_vel[0])
//     }

//     fn roll_rate_rads(&self) -> Option<f32> {
//         Some(self.physics.local_angular_vel[2])
//     }

//     fn steering_angle_rad(&self) -> f32 {
//         self.physics.steer_angle
//     }

//     fn throttle_norm(&self) -> f32 {
//         self.physics.gas.clamp(0.0, 1.0)
//     }

//     fn brake_norm(&self) -> f32 {
//         self.physics.brake.clamp(0.0, 1.0)
//     }

//     fn gear(&self) -> Option<i8> {
//         Some(self.physics.gear as i8)
//     }

//     fn wheel_speed_ms(&self) -> [f32; 4] {
//         // AC expose la vitesse angulaire en rad/s — on la convertit avec le rayon nominal
//         // TODO: rayon à paramétrer par voiture
//         const TYRE_RADIUS_M: f32 = 0.33;
//         self.physics.wheel_angular_speed.map(|w| w * TYRE_RADIUS_M)
//     }

//     fn suspension_travel_m(&self) -> Option<[f32; 4]> {
//         Some(self.physics.suspension_travel)
//     }

//     fn tyre_slip_ratio(&self) -> Option<[f32; 4]> {
//         Some(self.physics.wheel_slip)
//     }

//     fn tyre_load_n(&self) -> Option<[f32; 4]> {
//         Some(self.physics.wheel_load)
//     }

//     fn tyre_temp_celsius(&self) -> Option<[f32; 4]> {
//         Some(self.physics.tyre_core_temp)
//     }

//     fn tyre_pressure_kpa(&self) -> Option<[f32; 4]> {
//         Some(self.physics.wheels_pressure)
//     }

//     fn lap_distance_m(&self) -> Option<f32> {
//         self.lap_distance
//     }

//     fn simulator_id(&self) -> &'static str {
//         "assetto_corsa"
//     }
// }