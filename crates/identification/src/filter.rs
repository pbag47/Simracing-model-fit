use telemetry::TelemetrySample;

/// Critères de validité pour l'identification du modèle bicyclette linéaire.
/// Chaque critère correspond à une hypothèse du modèle.
#[derive(Debug, Clone)]
pub struct FilterCriteria {
    /// Accélération latérale maximale (g) — zone linéaire pneus
    pub max_lateral_accel_g: f64,
    /// Accélération longitudinale maximale en valeur absolue (g)
    /// — limite le couplage Fx/Fy et les transferts de charge dynamiques
    pub max_longitudinal_accel_g: f64,
    /// Vitesse minimale (m/s) — en dessous, le modèle bicyclette dégénère
    pub min_speed_ms: f64,
    /// Taux de variation maximal de l'accélération latérale (g/s)
    /// — filtre les transitoires où le régime permanent n'est pas atteint
    pub max_lateral_jerk_gs: f64,
    /// Angle de braquage minimal en valeur absolue (rad)
    /// — les lignes droites n'apportent rien à l'identification latérale
    pub min_steering_rad: f64,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            max_lateral_accel_g:      0.35,
            max_longitudinal_accel_g: 0.15,
            min_speed_ms:             40.0 / 3.6, // ~11 m/s
            max_lateral_jerk_gs:       0.5,
            min_steering_rad:          0.02,       // ~1.1°
        }
    }
}

impl FilterCriteria {
    /// Critères assouplis pour circuits avec dénivelé ou virages lents
    pub fn relaxed() -> Self {
        Self {
            max_lateral_accel_g:      0.45,
            max_longitudinal_accel_g: 0.25,
            min_speed_ms:             30.0 / 3.6,
            max_lateral_jerk_gs:       0.8,
            min_steering_rad:          0.02,
        }
    }
}

/// Raison pour laquelle un sample est rejeté
#[derive(Debug, Clone, PartialEq)]
pub enum RejectReason {
    LateralAccelTooHigh,
    LongitudinalAccelTooHigh,
    SpeedTooLow,
    LateralJerkTooHigh,
    SteeringTooSmall,
}

/// Statistiques de filtrage — utiles pour diagnostiquer
/// pourquoi trop peu de samples passent le filtre
#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    pub total:                   usize,
    pub accepted:                usize,
    pub rejected_lateral:        usize,
    pub rejected_longitudinal:   usize,
    pub rejected_speed:          usize,
    pub rejected_jerk:           usize,
    pub rejected_steering:       usize,
}

impl FilterStats {
    pub fn acceptance_rate(&self) -> f64 {
        if self.total == 0 { return 0.0; }
        self.accepted as f64 / self.total as f64
    }

    pub fn print_summary(&self) {
        println!("=== Filtrage des samples ===");
        println!("  Total          : {}", self.total);
        println!("  Acceptés       : {} ({:.1}%)",
            self.accepted, self.acceptance_rate() * 100.0);
        println!("  Rejetés ay     : {} ({:.1}%)",
            self.rejected_lateral,
            self.rejected_lateral as f64 / self.total as f64 * 100.0);
        println!("  Rejetés ax     : {} ({:.1}%)",
            self.rejected_longitudinal,
            self.rejected_longitudinal as f64 / self.total as f64 * 100.0);
        println!("  Rejetés v<min  : {} ({:.1}%)",
            self.rejected_speed,
            self.rejected_speed as f64 / self.total as f64 * 100.0);
        println!("  Rejetés jerk   : {} ({:.1}%)",
            self.rejected_jerk,
            self.rejected_jerk as f64 / self.total as f64 * 100.0);
        println!("  Rejetés δ<min  : {} ({:.1}%)",
            self.rejected_steering,
            self.rejected_steering as f64 / self.total as f64 * 100.0);
    }
}

pub struct SampleFilter;

impl SampleFilter {
    /// Filtre une séquence de samples selon les critères fournis.
    /// Retourne les indices des samples acceptés + les statistiques.
    pub fn filter<S: TelemetrySample>(
        samples: &[S],
        criteria: &FilterCriteria,
    ) -> (Vec<usize>, FilterStats) {
        let mut stats = FilterStats { total: samples.len(), ..Default::default() };
        let mut accepted = Vec::new();

        for (i, s) in samples.iter().enumerate() {
            let reason = Self::check_sample(s, samples, i, criteria);
            match reason {
                None => {
                    stats.accepted += 1;
                    accepted.push(i);
                }
                Some(RejectReason::LateralAccelTooHigh)     => stats.rejected_lateral      += 1,
                Some(RejectReason::LongitudinalAccelTooHigh) => stats.rejected_longitudinal += 1,
                Some(RejectReason::SpeedTooLow)             => stats.rejected_speed         += 1,
                Some(RejectReason::LateralJerkTooHigh)      => stats.rejected_jerk          += 1,
                Some(RejectReason::SteeringTooSmall)        => stats.rejected_steering      += 1,
            }
        }

        (accepted, stats)
    }

    fn check_sample<S: TelemetrySample>(
        s: &S,
        all: &[S],
        i: usize,
        c: &FilterCriteria,
    ) -> Option<RejectReason> {
        let ay = s.acceleration_g()[0].abs() as f64;
        let ax = s.acceleration_g()[1].abs() as f64;
        let v  = s.speed_ms() as f64;
        let steer = s.steering_angle_rad().abs() as f64;

        if v < c.min_speed_ms {
            return Some(RejectReason::SpeedTooLow);
        }
        if ay > c.max_lateral_accel_g {
            return Some(RejectReason::LateralAccelTooHigh);
        }
        if ax > c.max_longitudinal_accel_g {
            return Some(RejectReason::LongitudinalAccelTooHigh);
        }
        if steer < c.min_steering_rad {
            return Some(RejectReason::SteeringTooSmall);
        }

        // Calcul du jerk latéral (nécessite le sample précédent)
        if i > 0 {
            let prev = &all[i - 1];
            let dt_ms = s.timestamp_ms().saturating_sub(prev.timestamp_ms());
            if dt_ms > 0 {
                let dt_s  = dt_ms as f64 / 1000.0;
                let ay_prev = prev.acceleration_g()[0].abs() as f64;
                let jerk = (ay - ay_prev).abs() / dt_s;
                if jerk > c.max_lateral_jerk_gs {
                    return Some(RejectReason::LateralJerkTooHigh);
                }
            }
        }

        None
    }
}