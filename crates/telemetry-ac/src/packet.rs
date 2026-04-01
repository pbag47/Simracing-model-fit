

use serde::{Deserialize, Serialize};
use bytemuck::{Pod, Zeroable};


//
// HANDSHAKE
//
// ── Handshake (client → AC) ───────────────────────────────────────────────
// Envoyé 2 fois : d'abord operationId=0 (HANDSHAKE), puis operationId=1 (SUBSCRIBE_UPDATE)
// Exactement 12 octets : 3 × i32 little-endian
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct AcHandshakePacket {
    pub identifier:   i32,   // toujours 1
    pub version:      i32,   // toujours 1
    pub operation_id: i32,   // 0=HANDSHAKE, 1=SUBSCRIBE_UPDATE, 2=SUBSCRIBE_SPOT, 3=DISMISS
}

pub mod operation_id {
    pub const HANDSHAKE:        i32 = 0;
    pub const SUBSCRIBE_UPDATE: i32 = 1;
    pub const SUBSCRIBE_SPOT:   i32 = 2;
    pub const DISMISS:          i32 = 3;
}

impl AcHandshakePacket {
    pub fn handshake() -> Self {
        Self { identifier: 1, version: 1, operation_id: operation_id::HANDSHAKE }
    }
    pub fn subscribe_update() -> Self {
        Self { identifier: 1, version: 1, operation_id: operation_id::SUBSCRIBE_UPDATE }
    }
    pub fn subscribe_spot() -> Self {
        Self { identifier: 1, version: 1, operation_id: operation_id::SUBSCRIBE_SPOT }
    }
    pub fn dismiss() -> Self {
        Self { identifier: 1, version: 1, operation_id: operation_id::DISMISS }
    }
}

const _: () = assert!(std::mem::size_of::<AcHandshakePacket>() == 12);




//
// HANDSHAKE RESPONSE
//
// ── Handshake Response (AC → client) ─────────────────────────────────────
// 408 octets : carName[50] + driverName[50] + identifier(i32) + version(i32)
//            + trackName[50] + trackConfig[50]
// Les strings sont en ASCII null-terminated stockées dans des [u8; 50]
// Note : 50+50+4+4+50+50 = 208 octets. Les 200 octets restants sont du padding
// non documenté — probablement des champs réservés pour usage futur.
#[derive(Debug, Clone)]
pub struct AcHandshakeResponse {
    pub car_name:     String,
    pub driver_name:  String,
    pub identifier:   i32,
    pub version:      i32,
    pub track_name:   String,
    pub track_config: String,
}

impl AcHandshakeResponse {
    pub const SIZE: usize = 408;

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            car_name:     null_terminated_ascii(&data[0..50]),
            driver_name:  null_terminated_ascii(&data[50..100]),
            identifier:   i32::from_le_bytes(data[100..104].try_into().ok()?),
            version:      i32::from_le_bytes(data[104..108].try_into().ok()?),
            track_name:   null_terminated_ascii(&data[108..158]),
            track_config: null_terminated_ascii(&data[158..208]),  
            // bytes 208..408 : padding non documenté, ignoré
        })
    }
}

// Assertion invalide
// const _: () = assert!(std::mem::size_of::<AcHandshakeResponse>() == 408);



//
//  RTLAP
//
// ── RTLap (AC → client, après SUBSCRIBE_SPOT) ────────────────────────────
// Envoyé à chaque fin de tour (spot event) — 212 octets
// i32 + i32 + char[50] + char[50] + i32 = 8 + 50 + 50 + 4 = 112 octets
// Les 100 octets restants sont du padding non documenté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcRtLap {
    pub car_identifier_number: i32,
    pub lap:                   i32,
    pub driver_name:           String,
    pub car_name:              String,
    pub time_ms:               i32,
    // pub _pad1:                 [u8; 3],
}

impl AcRtLap {
    pub const SIZE: usize = 212;

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            car_identifier_number: i32::from_le_bytes(data[0..4].try_into().ok()?),
            lap:                   i32::from_le_bytes(data[4..8].try_into().ok()?),
            driver_name:           null_terminated_ascii(&data[8..58]),    
            car_name:              null_terminated_ascii(&data[58..108]),    
            time_ms:               i32::from_le_bytes(data[108..112].try_into().ok()?),
            // bytes 112..212 : padding, ignoré
        })
    }

    pub fn lap_time_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.time_ms.max(0) as u64)
    }
}

// Assertion invalide
// const _: () = assert!(std::mem::size_of::<AcRtLap>() == 212);



//
// CARINFO
//
// ── RTCarInfo (AC → client, après SUBSCRIBE_UPDATE) ──────────────────────
// Format officiel de la doc Remote Telemetry — ~328 octets
// Les bool AC sont des u8 (1 octet chacun)
#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, Serialize, Deserialize)]
pub struct AcRtCarInfo {
    pub identifier:               u8,

    pub _pad1:                    [u8; 3],

    pub size:                     i32,
    pub speed_kmh:                f32,
    pub speed_mph:                f32,
    pub speed_ms:                 f32,
    pub is_abs_enabled:           u8,
    pub is_abs_in_action:         u8,
    pub is_tc_in_action:          u8,
    pub is_tc_enabled:            u8,
    pub is_in_pit:                u8,
    pub is_engine_limiter_on:     u8,

    pub _pad2:                    [u8; 2],

    pub acc_g_vertical:           f32,
    pub acc_g_horizontal:         f32,
    pub acc_g_frontal:            f32,
    pub lap_time_ms:              i32,
    pub last_lap_ms:              i32,
    pub best_lap_ms:              i32,
    pub lap_count:                i32,
    pub gas:                      f32,
    pub brake:                    f32,
    pub clutch:                   f32,
    pub engine_rpm:               f32,
    pub steer:                    f32,
    pub gear:                     i32,
    pub cg_height:                f32,
    pub wheel_angular_speed:      [f32; 4],
    pub slip_angle:               [f32; 4],
    pub slip_angle_contact_patch: [f32; 4],
    pub slip_ratio:               [f32; 4],
    pub tyre_slip:                [f32; 4],
    pub nd_slip:                  [f32; 4],
    pub load:                     [f32; 4],
    pub dy:                       [f32; 4],
    pub mz:                       [f32; 4],
    pub tyre_dirty_level:         [f32; 4],
    pub camber_rad:               [f32; 4],
    pub tyre_radius:              [f32; 4],
    pub tyre_loaded_radius:       [f32; 4],
    pub suspension_height:        [f32; 4],
    pub car_position_normalized:  f32,
    pub car_slope:                f32,
    pub car_coordinates:          [f32; 3],
}

const _: () = assert!(std::mem::size_of::<AcRtCarInfo>() == 328);




// ── Utilitaire ────────────────────────────────────────────────────────────

fn null_terminated_ascii(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}