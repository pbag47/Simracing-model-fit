use serde::{Deserialize, Serialize};

// ── Chassis → Essieu ──────────────────────────────────────────────────────

/// Transferts de charge et moments transmis à chaque essieu
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChassisToAxle {
    /// Charge statique sur l'essieu (N)
    pub static_load: f64,
    /// Transfert de charge longitudinal (N) — positif vers l'avant
    pub longitudinal_transfer: f64,
    /// Moment de roulis transmis à l'essieu (N·m)
    pub roll_moment: f64,
    /// Accélération latérale au centre de gravité (g)
    pub acc_lateral_g: f64,
}

// ── Essieu → Suspension ───────────────────────────────────────────────────

/// Sollicitations transmises à chaque suspension (une roue)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AxleToSuspension {
    /// Charge verticale sur la roue (N)
    pub vertical_load: f64,
    /// Variation de charge due au transfert de roulis (N)
    pub load_transfer: f64,
}

// ── Suspension → Pneu ─────────────────────────────────────────────────────

/// État de la roue transmis au modèle pneu
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuspensionToTyre {
    /// Charge verticale sur le pneu (N)
    pub vertical_load: f64,
    /// Débattement de suspension (m) — utilisé pour le carrossage dynamique
    pub suspension_travel: f64,
    /// Carrossage résultant (rad)
    pub camber_rad: f64,
}

// ── Pneu → Châssis (remontée des efforts) ─────────────────────────────────

/// Efforts générés par un pneu, exprimés dans le repère roue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TyreForces {
    /// Force longitudinale (N) — traction/freinage
    pub fx: f64,
    /// Force latérale (N) — dérive
    pub fy: f64,
    /// Force verticale / charge (N)
    pub fz: f64,
    /// Moment d'autoalignement (N·m)
    pub mz: f64,
}

impl TyreForces {
    /// Effort résultant dans le plan (N)
    pub fn horizontal_magnitude(&self) -> f64 {
        (self.fx * self.fx + self.fy * self.fy).sqrt()
    }

    /// Coefficient d'utilisation de l'adhérence (0..1+)
    pub fn friction_usage(&self, mu: f64) -> f64 {
        if self.fz > 0.0 {
            self.horizontal_magnitude() / (mu * self.fz)
        } else {
            0.0
        }
    }
}

// ── Entrée pneu (depuis la dynamique véhicule) ────────────────────────────

/// Cinématique de la roue transmise au modèle pneu
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TyreKinematics {
    /// Angle de dérive (rad) — positif = nez de roue vers l'extérieur du virage
    pub slip_angle: f64,
    /// Taux de glissement longitudinal (adimensionnel, -1..+1)
    pub slip_ratio: f64,
    /// Charge verticale (N) — dupliqué depuis SuspensionToTyre pour commodité
    pub vertical_load: f64,
    /// Carrossage (rad)
    pub camber_rad: f64,
    /// Vitesse de rotation de la roue (rad/s)
    pub wheel_speed_rads: f64,
}