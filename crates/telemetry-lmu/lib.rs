// Le Mans Ultimate utilise le protocole rFactor 2 (même moteur).
// Le format est une Shared Memory Windows — on l'exposera via
// un reader fichier (replay) ou un bridge UDP pour les tests cross-platform.
// TODO itération suivante.

pub mod sample;
pub use sample::LmuSample;