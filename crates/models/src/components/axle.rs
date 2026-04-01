use serde::{Deserialize, Serialize};
use crate::traits::{ComponentModel, ModelError};
use crate::interfaces::{ChassisToAxle, AxleToSuspension};

/// Paramètres d'un essieu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxleParams {
    /// Raideur de la barre anti-roulis (N·m/rad)
    pub arb_stiffness: f64,
    /// Voie (m)
    pub track: f64,
}

impl Default for AxleParams {
    fn default() -> Self {
        Self { arb_stiffness: 8_000.0, track: 1.54 }
    }
}

/// Sorties de l'essieu : sollicitations vers chaque suspension
#[derive(Debug, Clone)]
pub struct AxleOutput {
    /// Suspension côté gauche (intérieur en virage à droite)
    pub left:  AxleToSuspension,
    /// Suspension côté droit
    pub right: AxleToSuspension,
}

pub struct AxleModel;

impl ComponentModel for AxleModel {
    type Params = AxleParams;
    type Input  = ChassisToAxle;
    type Output = AxleOutput;

    fn name(&self) -> &'static str { "Essieu" }

    fn default_params(&self) -> AxleParams { AxleParams::default() }

    fn validate_params(p: &AxleParams) -> Result<(), ModelError> {
        if p.arb_stiffness < 0.0 {
            return Err(ModelError::InvalidParameters(
                "arb_stiffness doit être >= 0".into()
            ));
        }
        if p.track <= 0.0 {
            return Err(ModelError::InvalidParameters("track doit être > 0".into()));
        }
        Ok(())
    }

    /// Répartit le transfert de charge latéral entre les deux roues.
    ///
    /// Le transfert latéral sur un essieu est la somme de :
    ///   - la part géométrique (via la rigidité en roulis de la suspension)
    ///   - la part de la barre anti-roulis
    ///
    /// Itération 1 (simplifié) : on modélise uniquement
    /// la contribution de la barre ARB au transfert latéral.
    /// La part géométrique sera ajoutée avec le modèle suspension complet.
    ///
    ///   ΔFz_arb = K_arb · φ / t
    ///
    /// où φ est l'angle de roulis estimé depuis le moment de roulis.
    fn evaluate(
        &self,
        input: &ChassisToAxle,
        p: &AxleParams,
    ) -> Result<AxleOutput, ModelError> {
        let half_load = input.static_load / 2.0;

        // Transfert latéral simplifié : m·ay·hcg / voie
        // (la répartition ARB vs géométrie sera détaillée à l'itération suivante)
        let delta_fz_lat = input.roll_moment / p.track;
        let arb_contribution = p.arb_stiffness / (p.arb_stiffness + 1.0)
            * delta_fz_lat; // placeholder — sera remplacé par le couplage suspension

        Ok(AxleOutput {
            left:  AxleToSuspension {
                vertical_load:  half_load - arb_contribution,
                load_transfer: -arb_contribution,
            },
            right: AxleToSuspension {
                vertical_load:  half_load + arb_contribution,
                load_transfer:  arb_contribution,
            },
        })
    }
}