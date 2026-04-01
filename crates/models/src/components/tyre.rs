use serde::{Deserialize, Serialize};
use crate::traits::{ComponentModel, ModelError};
use crate::interfaces::{TyreKinematics, TyreForces};

/// Paramètres du modèle pneu linéaire (itération 1).
///
/// Itération 1 : rigidité de dérive constante (linéaire en slip angle).
/// Itération 2 : Magic Formula de Pacejka (B, C, D, E).
/// Itération 3 : ajout de la sensibilité à la charge (Fz) et au carrossage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyreParams {
    /// Rigidité de dérive (N/rad) — remplacée par Pacejka à l'iter. 2
    pub cornering_stiffness: f64,
    /// Rigidité longitudinale (N/unité de slip ratio)
    pub longitudinal_stiffness: f64,
    /// Coefficient de friction maximal (adimensionnel)
    pub mu: f64,
}

impl Default for TyreParams {
    fn default() -> Self {
        Self {
            cornering_stiffness:    60_000.0,
            longitudinal_stiffness: 80_000.0,
            mu:                          1.2,
        }
    }
}

pub struct TyreModel;

impl ComponentModel for TyreModel {
    type Params = TyreParams;
    type Input  = TyreKinematics;
    type Output = TyreForces;

    fn name(&self) -> &'static str { "Pneu linéaire" }

    fn default_params(&self) -> TyreParams { TyreParams::default() }

    fn validate_params(p: &TyreParams) -> Result<(), ModelError> {
        if p.cornering_stiffness <= 0.0 {
            return Err(ModelError::InvalidParameters(
                "cornering_stiffness doit être > 0".into()
            ));
        }
        if p.mu <= 0.0 {
            return Err(ModelError::InvalidParameters("mu doit être > 0".into()));
        }
        Ok(())
    }

    /// Modèle pneu linéaire avec saturation par friction circle.
    ///
    /// Fy = Cf · α        (dérive latérale)
    /// Fx = Cx · κ        (traction/freinage)
    ///
    /// Saturation : si √(Fx²+Fy²) > μ·Fz, on normalise au cercle de friction.
    /// C'est la limite du modèle linéaire — Magic Formula gère ça naturellement.
    fn evaluate(
        &self,
        input: &TyreKinematics,
        p: &TyreParams,
    ) -> Result<TyreForces, ModelError> {
        let fz = input.vertical_load.max(0.0);

        // Efforts linéaires bruts
        let fy_raw = p.cornering_stiffness    * input.slip_angle;
        let fx_raw = p.longitudinal_stiffness * input.slip_ratio;

        // Saturation par friction circle
        let f_max  = p.mu * fz;
        let f_total = (fx_raw * fx_raw + fy_raw * fy_raw).sqrt();
        let (fx, fy) = if f_total > f_max && f_total > 0.0 {
            let scale = f_max / f_total;
            (fx_raw * scale, fy_raw * scale)
        } else {
            (fx_raw, fy_raw)
        };

        // Moment d'autoalignement simplifié (sera précisé avec Pacejka)
        let pneumatic_trail = 0.04; // m — valeur typique
        let mz = -fy * pneumatic_trail;

        Ok(TyreForces { fx, fy, fz, mz })
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ComponentModel;

    #[test]
    fn test_zero_slip_zero_forces() {
        let model = TyreModel;
        let p = TyreParams::default();
        let input = TyreKinematics {
            slip_angle: 0.0, slip_ratio: 0.0,
            vertical_load: 3000.0, camber_rad: 0.0,
            wheel_speed_rads: 100.0,
        };
        let out = model.evaluate(&input, &p).unwrap();
        assert!(out.fy.abs() < 1e-10);
        assert!(out.fx.abs() < 1e-10);
    }

    #[test]
    fn test_friction_circle_saturation() {
        let model = TyreModel;
        let p = TyreParams::default();
        // Slip énorme → doit saturer au cercle de friction
        let input = TyreKinematics {
            slip_angle: 0.5, slip_ratio: 0.5,
            vertical_load: 3000.0, camber_rad: 0.0,
            wheel_speed_rads: 100.0,
        };
        let out = model.evaluate(&input, &p).unwrap();
        let f_resultant = (out.fx * out.fx + out.fy * out.fy).sqrt();
        let f_max = p.mu * 3000.0;
        assert!(f_resultant <= f_max * 1.001,
            "friction circle dépassé : {f_resultant:.1} > {f_max:.1}");
    }

    #[test]
    fn test_chassis_load_distribution() {
        use crate::components::chassis::{ChassisModel, ChassisParams, ChassisInput};
        let model = ChassisModel;
        let p = ChassisParams::default();
        // En ligne droite sans accélération, la charge doit se répartir
        // selon l_front/l_rear
        let input = ChassisInput {
            acc_lateral_g: 0.0, acc_longitudinal_g: 0.0, speed_ms: 0.0
        };
        let out = model.evaluate(&input, &p).unwrap();
        let l = p.l_front + p.l_rear;
        let expected_front = p.mass * 9.81 * p.l_rear / l;
        let expected_rear  = p.mass * 9.81 * p.l_front / l;
        assert!((out.front.static_load - expected_front).abs() < 1.0);
        assert!((out.rear.static_load  - expected_rear ).abs() < 1.0);
    }
}