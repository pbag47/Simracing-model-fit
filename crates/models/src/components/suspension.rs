use serde::{Deserialize, Serialize};
use crate::traits::{ComponentModel, ModelError};
use crate::interfaces::{AxleToSuspension, SuspensionToTyre};

/// Paramètres d'une suspension (une roue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionParams {
    /// Raideur du ressort à la roue (N/m)
    pub spring_rate: f64,
    /// Coefficient d'amortissement à la roue (N·s/m)
    pub damper_rate: f64,
    /// Motion ratio (adimensionnel) — rapport débattement roue / débattement ressort
    /// ex: 0.7 signifie que le ressort se comprime de 0.7 mm par mm de débattement roue
    pub motion_ratio: f64,
    /// Raideur effective à la roue = spring_rate × motion_ratio²
    /// Calculée automatiquement, non identifiée directement
    pub ride_height_ref: f64,
}

impl Default for SuspensionParams {
    fn default() -> Self {
        Self {
            spring_rate:    25_000.0,
            damper_rate:     2_000.0,
            motion_ratio:        0.7,
            ride_height_ref:     0.08,
        }
    }
}

impl SuspensionParams {
    /// Raideur effective à la roue (N/m)
    pub fn wheel_rate(&self) -> f64 {
        self.spring_rate * self.motion_ratio * self.motion_ratio
    }
}

/// Entrée suspension : sollicitation depuis l'essieu + cinématique
#[derive(Debug, Clone)]
pub struct SuspensionInput {
    pub axle: AxleToSuspension,
    /// Vitesse de débattement (m/s) — positive = compression
    pub travel_rate: f64,
}

pub struct SuspensionModel;

impl ComponentModel for SuspensionModel {
    type Params = SuspensionParams;
    type Input  = SuspensionInput;
    type Output = SuspensionToTyre;

    fn name(&self) -> &'static str { "Suspension" }

    fn default_params(&self) -> SuspensionParams { SuspensionParams::default() }

    fn validate_params(p: &SuspensionParams) -> Result<(), ModelError> {
        if p.spring_rate <= 0.0 {
            return Err(ModelError::InvalidParameters("spring_rate doit être > 0".into()));
        }
        if p.damper_rate < 0.0 {
            return Err(ModelError::InvalidParameters("damper_rate doit être >= 0".into()));
        }
        if p.motion_ratio <= 0.0 || p.motion_ratio > 1.5 {
            return Err(ModelError::InvalidParameters(
                "motion_ratio doit être dans ]0, 1.5]".into()
            ));
        }
        Ok(())
    }

    /// Calcule le débattement et la charge transmise au pneu.
    ///
    /// Débattement statique : x_static = Fz / wheel_rate
    /// Effort amortisseur   : F_damper = damper_rate × ẋ × motion_ratio²
    /// Charge pneu          : Fz_tyre = Fz_static + ΔFz_transfert
    ///
    /// Itération 1 : modèle quasi-statique (sans dynamique de suspension).
    /// L'amortisseur sera intégré dynamiquement à l'itération suivante.
    fn evaluate(
        &self,
        input: &SuspensionInput,
        p: &SuspensionParams,
    ) -> Result<SuspensionToTyre, ModelError> {
        let wheel_rate = p.wheel_rate();
        let fz = input.axle.vertical_load;

        // Débattement quasi-statique
        let travel = if wheel_rate > 0.0 {
            fz / wheel_rate
        } else {
            0.0
        };

        // Carrossage simplifié (sera lié à la géométrie de suspension plus tard)
        let camber_rad = 0.0;

        Ok(SuspensionToTyre {
            vertical_load:     fz,
            suspension_travel: travel,
            camber_rad,
        })
    }
}