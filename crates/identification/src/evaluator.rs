use telemetry_core::TelemetrySample;
use models::{VehicleModel, VehicleState, VehicleInput};
use models::vehicle::bicycle::{BicycleModel, BicycleParams};

/// Résultat de l'évaluation du modèle sur un ensemble de samples
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Résidus [ay_prédit - ay_mesuré, yaw_prédit - yaw_mesuré] pour chaque sample
    pub residuals: Vec<[f64; 2]>,
    /// Variance Accounted For sur ay (0..1, 1 = parfait)
    pub vaf_lateral: f64,
    /// Variance Accounted For sur yaw rate
    pub vaf_yaw: f64,
    /// RMSE sur ay (g)
    pub rmse_lateral_g: f64,
    /// RMSE sur yaw rate (rad/s)
    pub rmse_yaw_rads: f64,
}

impl EvaluationResult {
    /// Qualité globale — moyenne des deux VAF.
    /// > 0.90 : bon, > 0.95 : très bon, > 0.98 : excellent
    pub fn quality_score(&self) -> f64 {
        (self.vaf_lateral + self.vaf_yaw) / 2.0
    }

    pub fn print_summary(&self) {
        println!("=== Qualité d'identification ===");
        println!("  VAF ay       : {:.3} ({:.1}%)", self.vaf_lateral, self.vaf_lateral * 100.0);
        println!("  VAF yaw      : {:.3} ({:.1}%)", self.vaf_yaw,     self.vaf_yaw     * 100.0);
        println!("  RMSE ay      : {:.4} g",        self.rmse_lateral_g);
        println!("  RMSE yaw     : {:.4} rad/s",    self.rmse_yaw_rads);
        println!("  Score global : {:.3}",           self.quality_score());
    }
}

pub struct ModelEvaluator;

impl ModelEvaluator {
    /// Évalue le modèle bicyclette sur les samples sélectionnés.
    /// Chaque sample est évalué indépendamment (pas d'intégration temporelle) :
    /// on calcule l'état prédit à partir de l'état mesuré et on compare.
    ///
    /// Cette approche "one-step" est plus robuste que l'intégration sur toute
    /// la session car elle évite l'accumulation d'erreurs.
    pub fn evaluate_bicycle<S: TelemetrySample>(
        samples: &[S],
        indices: &[usize],
        params: &BicycleParams,
    ) -> EvaluationResult {
        let model = BicycleModel;
        let mut residuals = Vec::with_capacity(indices.len());
        let mut ay_measured   = Vec::with_capacity(indices.len());
        let mut yaw_measured  = Vec::with_capacity(indices.len());
        let mut ay_predicted  = Vec::with_capacity(indices.len());
        let mut yaw_predicted = Vec::with_capacity(indices.len());

        for &i in indices {
            let s = &samples[i];

            // Pas de temps vers le sample suivant (ou 1/60s par défaut)
            let dt = if i + 1 < samples.len() {
                let dt_ms = samples[i + 1].timestamp_ms()
                    .saturating_sub(s.timestamp_ms());
                (dt_ms as f64 / 1000.0).clamp(0.001, 0.1)
            } else {
                1.0 / 60.0
            };

            let state = VehicleState::from_sample(s);
            let input = VehicleInput::from_sample(s, params.steering_ratio);

            match model.step(&state, &input, params, dt) {
                Ok(predicted) => {
                    let res_ay  = predicted.acc_lateral_g - state.acc_lateral_g;
                    let res_yaw = predicted.yaw_rate      - state.yaw_rate;
                    residuals.push([res_ay, res_yaw]);
                    ay_measured  .push(state.acc_lateral_g);
                    yaw_measured .push(state.yaw_rate);
                    ay_predicted .push(predicted.acc_lateral_g);
                    yaw_predicted.push(predicted.yaw_rate);
                }
                Err(e) => {
                    tracing::warn!("Erreur step sur sample {i} : {e}");
                }
            }
        }

        let vaf_lateral = vaf(&ay_measured,  &ay_predicted);
        let vaf_yaw     = vaf(&yaw_measured, &yaw_predicted);
        let rmse_lateral_g = rmse(&residuals.iter().map(|r| r[0]).collect::<Vec<_>>());
        let rmse_yaw_rads  = rmse(&residuals.iter().map(|r| r[1]).collect::<Vec<_>>());

        EvaluationResult { residuals, vaf_lateral, vaf_yaw, rmse_lateral_g, rmse_yaw_rads }
    }
}

/// Variance Accounted For : VAF = 1 - var(y_mes - y_pred) / var(y_mes)
/// 1.0 = prédiction parfaite, 0.0 = pas mieux que la moyenne
fn vaf(measured: &[f64], predicted: &[f64]) -> f64 {
    if measured.len() < 2 { return 0.0; }
    let mean_m = measured.iter().sum::<f64>() / measured.len() as f64;
    let var_m: f64 = measured.iter().map(|&y| (y - mean_m).powi(2)).sum::<f64>();
    if var_m < 1e-12 { return 1.0; } // signal constant → pas de variance à expliquer
    let var_err: f64 = measured.iter().zip(predicted)
        .map(|(&m, &p)| (m - p).powi(2)).sum::<f64>();
    (1.0 - var_err / var_m).clamp(0.0, 1.0)
}

fn rmse(residuals: &[f64]) -> f64 {
    if residuals.is_empty() { return 0.0; }
    (residuals.iter().map(|&r| r * r).sum::<f64>() / residuals.len() as f64).sqrt()
}