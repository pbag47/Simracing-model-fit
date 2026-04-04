use serde::{Deserialize, Serialize};
use crate::traits::{VehicleModel, ModelError};
use crate::state::{VehicleState, VehicleInput};


/// Paramètres physiques du modèle bicyclette linéaire.
///
/// Ce sont les grandeurs à identifier par le solveur.
/// Les distances L_f, L_r et la masse m peuvent être estimées
/// depuis les données constructeur et affinées par identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BicycleParams {
    /// Rigidité de dérive avant (N/rad) — typiquement 40 000..120 000 pour une voiture de route
    pub cornering_stiffness_front: f64,
    /// Rigidité de dérive arrière (N/rad)
    pub cornering_stiffness_rear: f64,
    /// Moment d'inertie en lacet (kg·m²) — typiquement 1 000..3 000 pour une voiture de route
    pub yaw_inertia: f64,
    /// Masse totale du véhicule (kg)
    pub mass: f64,
    /// Distance centre de gravité → essieu avant (m)
    pub l_front: f64,
    /// Distance centre de gravité → essieu arrière (m)
    pub l_rear: f64,
    /// Rapport de direction volant → roue (adimensionnel, ex: 15.0)
    /// Utilisé pour convertir l'angle volant en angle de braquage roue.
    /// Note : déjà appliqué dans VehicleInput::from_sample — mis ici
    /// pour traçabilité et pour permettre son identification éventuelle.
    pub steering_ratio: f64,
}

impl BicycleParams {
    /// Empattement total
    pub fn wheelbase(&self) -> f64 {
        self.l_front + self.l_rear
    }

    /// Facteur de sous-virage K (rad·s²/m²).
    /// K > 0 → sous-vireur, K < 0 → survireur, K = 0 → neutre
    pub fn understeer_gradient(&self) -> f64 {
        let cf = self.cornering_stiffness_front;
        let cr = self.cornering_stiffness_rear;
        let m  = self.mass;
        let lf = self.l_front;
        let lr = self.l_rear;
        (m / self.wheelbase()) * (lr / cr - lf / cf)
    }

    /// Vitesse critique (survireur uniquement, K < 0).
    /// Au-delà, le véhicule est instable en lacet.
    pub fn critical_speed(&self) -> Option<f64> {
        let k = self.understeer_gradient();
        if k < 0.0 {
            Some((-self.wheelbase() / k).sqrt())
        } else {
            None // sous-vireur → pas de vitesse critique
        }
    }

    /// Gain en lacet stationnaire ψ̇/δ_volant à vitesse v (rad/s / rad)
    pub fn yaw_rate_gain(&self, v: f64) -> f64 {
        let l = self.wheelbase();
        let k = self.understeer_gradient();
        (v / l) / (1.0 + k * v * v)
    }
}

impl Default for BicycleParams {
    /// Valeurs typiques pour une voiture de route compacte (~1200 kg).
    /// Bon point de départ pour l'initialisation du solveur.
    fn default() -> Self {
        Self {
            cornering_stiffness_front: 60_000.0,
            cornering_stiffness_rear:  55_000.0,
            yaw_inertia:               1_500.0,
            mass:                      1_200.0,
            l_front:                   1.05,
            l_rear:                    1.55,
            steering_ratio:            15.0,
        }
    }
}

/// Le modèle bicyclette linéaire.
///
/// Hypothèses :
/// - Angles de dérive faibles (sin α ≈ α, cos α ≈ 1)
/// - Pas de transfert de charge (charge statique constante)
/// - Rigidités de dérive constantes (linéaires)
/// - Dynamique longitudinale découplée (vx imposée par la télémétrie)
///
/// Ces hypothèses sont valides pour :
/// - Vitesses modérées (< 150 km/h selon la voiture)
/// - Manœuvres peu aggressives (ay < ~0.6g)
/// - Voitures à faible appui aéro
///
/// Elles deviennent limites pour :
/// - Fortes accélérations latérales (pneus en régime non-linéaire)
/// - Voitures à appui aéro (charge variable avec la vitesse)
/// → On passera alors au modèle 7-DDL avec Magic Formula
pub struct BicycleModel;

impl VehicleModel for BicycleModel {
    type Params = BicycleParams;

    // Ajouter observable_names() à l'impl VehicleModel :
    fn observable_names(&self) -> &'static [&'static str] {
        &["acc_laterale_g", "yaw_rate_rads"]
    }

    fn name(&self) -> &'static str {
        "Bicyclette linéaire"
    }



    /// Intègre les équations du mouvement du modèle bicyclette.
    ///
    /// Équations (repère véhicule) :
    ///   m·(vy̋ + vx·ψ̇) = Fy_f + Fy_r
    ///   Iz·ψ̈          = Lf·Fy_f - Lr·Fy_r
    ///
    /// Angles de dérive :
    ///   α_f = δ - (vy + Lf·ψ̇) / vx
    ///   α_r =   - (vy - Lr·ψ̇) / vx
    ///
    /// Forces latérales (linéaires) :
    ///   Fy_f = Cf · α_f
    ///   Fy_r = Cr · α_r
    ///
    /// Intégration : Euler explicite (suffisant pour identification,
    /// on passera à RK4 pour la prédiction de temps au tour)
    fn step(
        &self,
        state: &VehicleState,
        input: &VehicleInput,
        p: &BicycleParams,
        dt: f64,
    ) -> Result<VehicleState, ModelError> {
        if dt <= 0.0 {
            return Err(ModelError::InvalidTimestep);
        }

        let vx = state.vx.max(0.5); // évite la division par zéro à vitesse nulle
        let vy = state.vy;
        let r  = state.yaw_rate;    // ψ̇
        let delta = input.steering_wheel_rad / p.steering_ratio; // déjà en angle roue

        let cf = p.cornering_stiffness_front;
        let cr = p.cornering_stiffness_rear;
        let lf = p.l_front;
        let lr = p.l_rear;
        let m  = p.mass;
        let iz = p.yaw_inertia;

        // Angles de dérive (rad)
        let alpha_f = delta - (vy + lf * r) / vx;
        let alpha_r =       - (vy - lr * r) / vx;

        // Forces latérales (N)
        let fy_f = cf * alpha_f;
        let fy_r = cr * alpha_r;

        // Dérivées
        let dvy_dt = (fy_f + fy_r) / m - vx * r;
        let dr_dt  = (lf * fy_f - lr * fy_r) / iz;

        // Intégration Euler
        let vy_new = vy + dvy_dt * dt;
        let r_new  = r  + dr_dt  * dt;

        // Accélération latérale prédite (g) — observable principale pour les résidus
        let acc_lat_predicted = (fy_f + fy_r) / (m * 9.81);

        Ok(VehicleState {
            vx:               state.vx, // vx imposée par la télémétrie
            vy:               vy_new,
            yaw_rate:         r_new,
            acc_lateral_g:    acc_lat_predicted,
            acc_longitudinal_g: state.acc_longitudinal_g, // non modélisée ici
            x:                None,
            y:                None,
            heading:          None,
            suspension_travel: None,
            tyre_load:        None,
            tyre_slip_angle:  None,
        })
    }

    /// Résidus entre état prédit et état mesuré.
    ///
    /// On observe deux grandeurs mesurables depuis la télémétrie :
    /// - L'accélération latérale (directement mesurée par l'accéléromètre AC)
    /// - Le taux de lacet (estimé depuis ay/vx dans notre cas,
    ///   mesuré directement si le sim l'expose)
    ///
    /// Ce sont les deux grandeurs que le solveur minimisera.
    fn residuals(
        &self,
        predicted: &VehicleState,
        measured: &VehicleState,
    ) -> Vec<f64> {
        vec![
            predicted.acc_lateral_g - measured.acc_lateral_g,
            predicted.yaw_rate      - measured.yaw_rate,
        ]
    }
}