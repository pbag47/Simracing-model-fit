use nalgebra::Vector3;
use telemetry_core::TelemetrySample;
use crate::packet::AcPhysicsPacket;

/// Sample normalisé construit depuis un AcPhysicsPacket
#[derive(Debug, Clone)]
pub struct AcSample {
    pub timestamp_ms: u64,
    pub physics: AcPhysicsPacket,
    pub lap_distance: Option<f32>,
}

impl AcSample {
    pub fn new(physics: AcPhysicsPacket, timestamp_ms: u64) -> Self {
        Self { timestamp_ms, physics, lap_distance: None }
    }
}

impl TelemetrySample for AcSample {
    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    fn speed_ms(&self) -> f32 {
        self.physics.speed_kmh / 3.6
    }

    fn velocity_world(&self) -> Option<Vector3<f32>> {
        let v = self.physics.velocity;
        Some(Vector3::new(v[0], v[1], v[2]))
    }

    fn acceleration_g(&self) -> Vector3<f32> {
        let a = self.physics.acc_g;
        // AC : acc_g[0]=lateral, [1]=longitudinal, [2]=vertical
        Vector3::new(a[0], a[1], a[2])
    }

    fn yaw_rate_rads(&self) -> f32 {
        // local_angular_vel : [pitch, yaw, roll] en rad/s
        self.physics.local_angular_vel[1]
    }

    fn pitch_rate_rads(&self) -> Option<f32> {
        Some(self.physics.local_angular_vel[0])
    }

    fn roll_rate_rads(&self) -> Option<f32> {
        Some(self.physics.local_angular_vel[2])
    }

    fn steering_angle_rad(&self) -> f32 {
        self.physics.steer_angle
    }

    fn throttle_norm(&self) -> f32 {
        self.physics.gas.clamp(0.0, 1.0)
    }

    fn brake_norm(&self) -> f32 {
        self.physics.brake.clamp(0.0, 1.0)
    }

    fn gear(&self) -> Option<i8> {
        Some(self.physics.gear as i8)
    }

    fn wheel_speed_ms(&self) -> [f32; 4] {
        // AC expose la vitesse angulaire en rad/s — on la convertit avec le rayon nominal
        // TODO: rayon à paramétrer par voiture
        const TYRE_RADIUS_M: f32 = 0.33;
        self.physics.wheel_angular_speed.map(|w| w * TYRE_RADIUS_M)
    }

    fn suspension_travel_m(&self) -> Option<[f32; 4]> {
        Some(self.physics.suspension_travel)
    }

    fn tyre_slip_ratio(&self) -> Option<[f32; 4]> {
        Some(self.physics.wheel_slip)
    }

    fn tyre_load_n(&self) -> Option<[f32; 4]> {
        Some(self.physics.wheel_load)
    }

    fn tyre_temp_celsius(&self) -> Option<[f32; 4]> {
        Some(self.physics.tyre_core_temp)
    }

    fn tyre_pressure_kpa(&self) -> Option<[f32; 4]> {
        Some(self.physics.wheels_pressure)
    }

    fn lap_distance_m(&self) -> Option<f32> {
        self.lap_distance
    }

    fn simulator_id(&self) -> &'static str {
        "assetto_corsa"
    }
}