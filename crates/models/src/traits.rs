use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Paramètres invalides : {0}")]
    InvalidParameters(String),

    #[error("État invalide : {0}")]
    InvalidState(String),

    #[error("Pas de temps nul ou négatif")]
    InvalidTimestep,

    #[error("Entrée hors domaine de validité : {0}")]
    OutOfDomain(String),
}

/// Contrat d'un composant physique élémentaire.
///
/// Un composant est une boîte qui reçoit des sollicitations (`Input`)
/// et produit des efforts ou moments (`Output`).
/// Il ne connaît pas le temps — il est stationnaire (algébrique).
/// L'intégration temporelle appartient au `VehicleModel`.
///
/// Exemples :
///   TyreModel    : Input=(charge, slip_angle, slip_ratio) → Output=(Fx, Fy, Mz)
///   SuspensionModel : Input=(débattement, vitesse_débattement) → Output=(Fz)
///   AxleModel    : Input=(Fz_gauche, Fz_droite, roulis) → Output=(ΔFz, Mx)
///   ChassisModel : Input=(accélérations) → Output=(transferts de charge)
pub trait ComponentModel: Send + Sync {
    /// Paramètres physiques identifiables de ce composant
    type Params: Clone + Serialize + for<'de> Deserialize<'de>;
    /// Sollicitations reçues par le composant
    type Input;
    /// Efforts / moments produits par le composant
    type Output;

    fn name(&self) -> &'static str;
    fn default_params(&self) -> Self::Params;

    /// Vérifie la cohérence physique des paramètres
    fn validate_params(params: &Self::Params) -> Result<(), ModelError>;

    /// Calcul stationnaire : sollicitations → efforts
    fn evaluate(
        &self,
        input: &Self::Input,
        params: &Self::Params,
    ) -> Result<Self::Output, ModelError>;
}

/// Contrat d'un modèle véhicule complet.
///
/// Un modèle véhicule orchestre l'appel aux composants,
/// propage les efforts entre eux, et intègre les équations du mouvement.
///
/// Deux familles d'implémentation :
///   - Boîte noire  : BicycleModel — paramètres globaux, pas de composants
///   - Assemblage   : FullVehicleModel — compose des ComponentModel
pub trait VehicleModel: Send + Sync {
    type Params: Clone + Serialize + for<'de> Deserialize<'de>;
    fn name(&self) -> &'static str;

    /// Intègre l'état sur un pas de temps `dt` (secondes).
    fn step(
        &self,
        state: &crate::state::VehicleState,
        input: &crate::state::VehicleInput,
        p: &Self::Params,
        dt: f64,
    ) -> Result<crate::state::VehicleState, ModelError>;

    /// Résidus entre état prédit et état mesuré.
    /// Le solveur minimisera le vecteur retourné.
    fn residuals(
        &self,
        predicted: &crate::state::VehicleState,
        measured:  &crate::state::VehicleState,
    ) -> Vec<f64>;

    /// Canaux observables utilisés dans les résidus, dans le même ordre.
    /// Utile pour afficher les résidus par nom dans le viewer.
    fn observable_names(&self) -> &'static [&'static str];
}