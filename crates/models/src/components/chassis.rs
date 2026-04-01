use serde::{Deserialize, Serialize};
use crate::traits::{ComponentModel, ModelError};
use crate::interfaces::ChassisToAxle;

/// Sollicitations reçues par le châssis (depuis la dynamique véhicule)
#[derive(Debug, Clone)]
pub struct ChassisInput {
    pub acc_lateral_g:      f64,
    pub acc_longitudinal_g: f64,
    pub speed_ms:           f64,
}

/// Paramètres du châssis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChassisParams {
    /// Masse totale (kg)
    pub mass: f64,
    /// Moment d'inertie en lacet (kg·m²)
    pub yaw_inertia: f64,
    /// Distance CdG → essieu avant (m)
    pub l_front: f64,
    /// Distance CdG → essieu arrière (m)
    pub l_rear: f64,
    /// Hauteur du centre de gravité (m)
    pub cg_height: f64,
    /// Voie avant (m) — distance entre centres roues
    pub track_front: f64,
    /// Voie arrière (m)
    pub track_rear: f64,
}

impl Default for ChassisParams {
    fn default() -> Self {
        Self {
            mass:        1200.0,
            yaw_inertia: 1500.0,
            l_front:     1.05,
            l_rear:      1.55,
            cg_height:   0.45,
            track_front: 1.55,
            track_rear:  1.52,
        }
    }
}

/// Sorties du châssis : transferts de charge vers chaque essieu
#[derive(Debug, Clone)]
pub struct ChassisOutput {
    pub front: ChassisToAxle,
    pub rear:  ChassisToAxle,
}

pub struct ChassisModel;

impl ComponentModel for ChassisModel {
    type Params = ChassisParams;
    type Input  = ChassisInput;
    type Output = ChassisOutput;

    fn name(&self) -> &'static str { "Châssis" }

    fn default_params(&self) -> ChassisParams { ChassisParams::default() }

    fn validate_params(p: &ChassisParams) -> Result<(), ModelError> {
        if p.mass <= 0.0 {
            return Err(ModelError::InvalidParameters("masse doit être > 0".into()));
        }
        if p.l_front <= 0.0 || p.l_rear <= 0.0 {
            return Err(ModelError::InvalidParameters("l_front et l_rear doivent être > 0".into()));
        }
        if p.cg_height < 0.0 {
            return Err(ModelError::InvalidParameters("cg_height doit être >= 0".into()));
        }
        Ok(())
    }

    /// Calcule les transferts de charge longitudinaux et latéraux.
    ///
    /// Transfert longitudinal (freinage/accélération) :
    ///   ΔFz_long = m · ax · hcg / L
    ///
    /// Transfert latéral total (réparti entre essieux par les barres ARB) :
    ///   ΔFz_lat = m · ay · hcg / voie
    ///
    /// La répartition avant/arrière du transfert latéral
    /// est calculée dans les AxleModel (via les barres ARB).
    fn evaluate(
        &self,
        input: &ChassisInput,
        p: &ChassisParams,
    ) -> Result<ChassisOutput, ModelError> {
        let g     = 9.81;
        let l     = p.l_front + p.l_rear;
        let w     = p.mass * g;

        // Charges statiques
        let fz_front_static = w * p.l_rear  / l;
        let fz_rear_static  = w * p.l_front / l;

        // Transfert longitudinal
        let delta_fz_long = p.mass * input.acc_longitudinal_g * g * p.cg_height / l;

        // Moment de roulis total (réparti par les essieux)
        let roll_moment_total = p.mass * input.acc_lateral_g * g * p.cg_height;

        Ok(ChassisOutput {
            front: ChassisToAxle {
                static_load:            fz_front_static - delta_fz_long,
                longitudinal_transfer:  delta_fz_long,
                roll_moment:            roll_moment_total, // sera réparti par AxleModel
                acc_lateral_g:          input.acc_lateral_g,
            },
            rear: ChassisToAxle {
                static_load:            fz_rear_static + delta_fz_long,
                longitudinal_transfer: -delta_fz_long,
                roll_moment:            roll_moment_total,
                acc_lateral_g:          input.acc_lateral_g,
            },
        })
    }
}