use bytemuck::{Pod, Zeroable};

/// Paquet Physics d'Assetto Corsa — 328 octets
/// Émis ~60 Hz par le plugin UDP
#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, Serialize, Deserialize)] 
pub struct AcPhysicsPacket {
    pub packet_id:          i32,
    pub gas:                f32,
    pub brake:              f32,
    pub fuel:               f32,
    pub gear:               i32,
    pub rpms:               i32,
    pub steer_angle:        f32,
    pub speed_kmh:          f32,
    pub velocity:           [f32; 3],       // world X, Y, Z  (m/s)
    pub acc_g:              [f32; 3],       // lateral, longitudinal, vertical (g)
    pub wheel_slip:         [f32; 4],       // FL FR RL RR
    pub wheel_load:         [f32; 4],
    pub wheels_pressure:    [f32; 4],       // kPa
    pub wheel_angular_speed:[f32; 4],       // rad/s
    pub tyre_wear:          [f32; 4],
    pub tyre_dirt_level:    [f32; 4],
    pub tyre_core_temp:     [f32; 4],       // °C
    pub camber_rad:         [f32; 4],
    pub suspension_travel:  [f32; 4],       // mètres
    pub drs:                f32,
    pub tc:                 f32,
    pub heading:            f32,            // rad
    pub pitch:              f32,            // rad
    pub roll:               f32,            // rad
    pub cg_height:          f32,
    pub car_damage:         [f32; 5],
    pub number_of_tyres_out:i32,
    pub pit_limiter_on:     i32,
    pub abs:                f32,
    pub kers_charge:        f32,
    pub kers_input:         f32,
    pub auto_shifter_on:    i32,
    pub ride_height:        [f32; 2],       // FL+FR avg, RL+RR avg
    pub turbo_boost:        f32,
    pub ballast:            f32,
    pub air_density:        f32,
    pub air_temp:           f32,
    pub road_temp:          f32,
    pub local_angular_vel:  [f32; 3],       // pitch, yaw, roll rate (rad/s)
    pub final_ff:           f32,
    pub performance_meter:  f32,
    pub engine_brake:       i32,
    pub ers_recovery_level: i32,
    pub ers_power_level:    i32,
    pub ers_heat_charging:  i32,
    pub ers_is_charging:    i32,
    pub kers_current_kj:    f32,
    pub drs_available:      i32,
    pub drs_enabled:        i32,
    pub brake_bias:         f32,
}

// Vérification statique de la taille au compile-time
const _: () = assert!(
    std::mem::size_of::<AcPhysicsPacket>() == 328,
    "AcPhysicsPacket: taille inattendue"
);

/// Paquet Graphics d'Assetto Corsa — informations session
#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct AcGraphicsPacket {
    pub packet_id:          i32,
    pub ac_status:          i32,    // 0=OFF 1=REPLAY 2=LIVE 3=PAUSE
    pub ac_session_type:    i32,
    pub current_time:       [u16; 15],  // UTF-16 LE
    pub last_time:          [u16; 15],
    pub best_time:          [u16; 15],
    pub split:              [u16; 15],
    pub completed_laps:     i32,
    pub position:           i32,
    pub i_current_time:     i32,    // ms
    pub i_last_time:        i32,    // ms
    pub i_best_time:        i32,    // ms
    pub session_time_left:  f32,
    pub distance_traveled:  f32,
    pub is_in_pit:          i32,
    pub current_sector_index: i32,
    pub last_sector_time:   i32,
    pub number_of_laps:     i32,
    pub tyre_compound:      [u16; 33],
    pub replay_time_multiplier: f32,
    pub normalized_car_position: f32, // 0..1 sur le tour
    pub car_coordinates:    [f32; 3],
}