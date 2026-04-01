use telemetry_ac::AcSample;
use telemetry_core::TelemetrySample;

/// Un canal = une série temporelle extraite de la session
#[derive(Clone)]
pub struct Signal {
    pub name:   &'static str,
    pub unit:   &'static str,
    /// Temps relatif depuis le début de la session, en secondes
    pub times:  Vec<f64>,
    pub values: Vec<f64>,
}

impl Signal {
    fn new(name: &'static str, unit: &'static str) -> Self {
        Self { name, unit, times: Vec::new(), values: Vec::new() }
    }

    pub fn push(&mut self, t: f64, v: f64) {
        self.times.push(t);
        self.values.push(v);
    }

    pub fn min(&self) -> f64 {
        self.values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    pub fn max(&self) -> f64 {
        self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
}

/// Tous les canaux extraits d'une session AC
pub struct SessionSignals {
    pub duration_s: f64,
    pub signals: Vec<Signal>,
}

impl SessionSignals {
    pub fn from_ac_samples(samples: &[AcSample]) -> Self {
        let t0 = samples.first().map(|s| s.timestamp_ms()).unwrap_or(0) as f64;

        // Déclare ici tous les canaux qu'on veut exposer
        let mut speed       = Signal::new("Vitesse",          "km/h");
        let mut throttle    = Signal::new("Gaz",              "%");
        let mut brake       = Signal::new("Frein",            "%");
        let mut steer       = Signal::new("Braquage",         "rad");
        let mut acc_lat     = Signal::new("Acc. latérale",    "g");
        let mut acc_lon     = Signal::new("Acc. longit.",     "g");
        let mut yaw_rate    = Signal::new("Lacet",            "rad/s");
        let mut gear        = Signal::new("Rapport",          "");
        let mut rpm         = Signal::new("Régime",           "tr/min");
        let mut susp_fl     = Signal::new("Susp. FL",         "m");
        let mut susp_fr     = Signal::new("Susp. FR",         "m");
        let mut susp_rl     = Signal::new("Susp. RL",         "m");
        let mut susp_rr     = Signal::new("Susp. RR",         "m");
        let mut slip_fl     = Signal::new("Slip ratio FL",    "");
        let mut slip_fr     = Signal::new("Slip ratio FR",    "");
        let mut slip_rl     = Signal::new("Slip ratio RL",    "");
        let mut slip_rr     = Signal::new("Slip ratio RR",    "");
        let mut load_fl     = Signal::new("Charge FL",        "N");
        let mut load_fr     = Signal::new("Charge FR",        "N");
        let mut load_rl     = Signal::new("Charge RL",        "N");
        let mut load_rr     = Signal::new("Charge RR",        "N");
        let mut ws_fl       = Signal::new("Vit. roue FL",     "m/s");
        let mut ws_fr       = Signal::new("Vit. roue FR",     "m/s");
        let mut ws_rl       = Signal::new("Vit. roue RL",     "m/s");
        let mut ws_rr       = Signal::new("Vit. roue RR",     "m/s");

        for s in samples {
            let t = (s.timestamp_ms() as f64 - t0) / 1000.0;
            let a = s.acceleration_g();
            let susp = s.suspension_travel_m().unwrap_or([0.0; 4]);
            let slip = s.tyre_slip_ratio().unwrap_or([0.0; 4]);
            let load = s.tyre_load_n().unwrap_or([0.0; 4]);
            let ws   = s.wheel_speed_ms();

            speed   .push(t, s.speed_ms() as f64 * 3.6);
            throttle.push(t, s.throttle_norm() as f64 * 100.0);
            brake   .push(t, s.brake_norm() as f64 * 100.0);
            steer   .push(t, s.steering_angle_rad() as f64);
            acc_lat .push(t, a[0] as f64);
            acc_lon .push(t, a[1] as f64);
            yaw_rate.push(t, s.yaw_rate_rads() as f64);
            gear    .push(t, s.gear().unwrap_or(0) as f64);
            rpm     .push(t, s.car_info.engine_rpm as f64);
            susp_fl .push(t, susp[0] as f64);
            susp_fr .push(t, susp[1] as f64);
            susp_rl .push(t, susp[2] as f64);
            susp_rr .push(t, susp[3] as f64);
            slip_fl .push(t, slip[0] as f64);
            slip_fr .push(t, slip[1] as f64);
            slip_rl .push(t, slip[2] as f64);
            slip_rr .push(t, slip[3] as f64);
            load_fl .push(t, load[0] as f64);
            load_fr .push(t, load[1] as f64);
            load_rl .push(t, load[2] as f64);
            load_rr .push(t, load[3] as f64);
            ws_fl   .push(t, ws[0] as f64);
            ws_fr   .push(t, ws[1] as f64);
            ws_rl   .push(t, ws[2] as f64);
            ws_rr   .push(t, ws[3] as f64);
        }

        let duration_s = samples.last()
            .map(|s| (s.timestamp_ms() as f64 - t0) / 1000.0)
            .unwrap_or(0.0);

        SessionSignals {
            duration_s,
            signals: vec![
                speed, throttle, brake, steer,
                acc_lat, acc_lon, yaw_rate, gear, rpm,
                susp_fl, susp_fr, susp_rl, susp_rr,
                slip_fl, slip_fr, slip_rl, slip_rr,
                load_fl, load_fr, load_rl, load_rr,
                ws_fl, ws_fr, ws_rl, ws_rr,
            ],
        }
    }
}