use crate::TelemetrySample;

/// Une session = une liste ordonnée de samples + métadonnées.
pub struct Session<S: TelemetrySample> {
    pub samples: Vec<S>,
    pub car_name: Option<String>,
    pub track_name: Option<String>,
    pub simulator: &'static str,
}

impl<S: TelemetrySample> Session<S> {
    pub fn new(simulator: &'static str) -> Self {
        Self {
            samples: Vec::new(),
            car_name: None,
            track_name: None,
            simulator,
        }
    }

    pub fn push(&mut self, sample: S) {
        self.samples.push(sample);
    }

    pub fn duration_s(&self) -> f64 {
        match (self.samples.first(), self.samples.last()) {
            (Some(first), Some(last)) => {
                (last.timestamp_ms() - first.timestamp_ms()) as f64 / 1000.0
            }
            _ => 0.0,
        }
    }

    pub fn sample_rate_hz(&self) -> Option<f64> {
        if self.samples.len() < 2 {
            return None;
        }
        let dt_ms = self.samples[1].timestamp_ms() - self.samples[0].timestamp_ms();
        if dt_ms == 0 { return None; }
        Some(1000.0 / dt_ms as f64)
    }
}