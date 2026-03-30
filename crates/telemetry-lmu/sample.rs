use nalgebra::Vector3;
use telemetry_core::TelemetrySample;

/// Placeholder — sera rempli avec le format rF2/LMU Shared Memory
#[derive(Debug, Clone)]
pub struct LmuSample {
    pub timestamp_ms: u64,
    pub speed_ms: f32,
    pub acc_g: [f32; 3],
    pub yaw_rate: f32,
    pub steering_rad: f32,
    pub throttle: f32,
    pub brake: f32,
    pub wheel_speed: [f32; 4],
}

impl TelemetrySample for LmuSample {
    fn timestamp_ms(&self) -> u64 { self.timestamp_ms }
    fn speed_ms(&self) -> f32 { self.speed_ms }
    fn acceleration_g(&self) -> Vector3<f32> {
        Vector3::new(self.acc_g[0], self.acc_g[1], self.acc_g[2])
    }
    fn yaw_rate_rads(&self) -> f32 { self.yaw_rate }
    fn steering_angle_rad(&self) -> f32 { self.steering_rad }
    fn throttle_norm(&self) -> f32 { self.throttle }
    fn brake_norm(&self) -> f32 { self.brake }
    fn wheel_speed_ms(&self) -> [f32; 4] { self.wheel_speed }
    fn simulator_id(&self) -> &'static str { "le_mans_ultimate" }
}