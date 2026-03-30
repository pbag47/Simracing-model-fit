use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use telemetry_ac::{AcUdpReader, AcSample};
use telemetry_core::TelemetrySample;
use session_store::{SessionStore, format::ChannelManifest};


#[derive(Parser)]
#[command(name = "simracing-fit", about = "Identification de modèles véhicule à partir de télémétrie")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enregistre une session AC en UDP et la sauvegarde dans un fichier .srf
    RecordAc {
        #[arg(long, default_value = "0.0.0.0:9996")]
        addr: String,
        #[arg(long, default_value_t = 3600)]
        max_samples: usize,
        /// Fichier de sortie (défaut : session_<timestamp>.srf)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Affiche les métadonnées d'un fichier .srf sans charger les samples
    Info {
        path: String,
    },
    /// Charge et rejoue une session .srf (affiche les N premiers samples)
    Replay {
        path: String,
        #[arg(long, default_value_t = 10)]
        samples: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("simracing_fit=debug".parse()?)
            .add_directive("telemetry_ac=debug".parse()?)
            .add_directive("session_store=debug".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::RecordAc { addr, max_samples, output } => {
            let reader = AcUdpReader::bind(&addr).await?;
            let session = reader.record_session(max_samples).await?;

            // Nom de fichier par défaut : session_<timestamp>.srf
            let out_path = output.unwrap_or_else(|| {
                let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                format!("session_{ts}.srf")
            });

            // Manifest des canaux disponibles dans AC
            let channels = ChannelManifest {
                suspension_travel: true,
                tyre_slip: true,
                tyre_load: true,
                tyre_temp: true,
                tyre_pressure: true,
                lap_distance: false, // rempli via paquet Graphics — TODO
                gear: true,
                position: false,
            };

            SessionStore::save(&session, &out_path, channels)?;

            println!("Session sauvegardée → {out_path}");
            println!("  {} samples | {:.1}s | {:.0} Hz",
                session.samples.len(),
                session.duration_s(),
                session.sample_rate_hz().unwrap_or(0.0)
            );
        }

        Commands::Info { path } => {
            let meta = SessionStore::read_metadata(&path)?;
            println!("=== {} ===", path);
            println!("  Simulateur   : {}", meta.simulator);
            println!("  Voiture      : {}", meta.car.as_deref().unwrap_or("inconnue"));
            println!("  Circuit      : {}", meta.track.as_deref().unwrap_or("inconnu"));
            println!("  Enregistré   : {}", meta.recorded_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("  Samples      : {}", meta.sample_count);
            println!("  Durée        : {:.1}s", meta.duration_s);
            println!("  Fréquence    : {:.0} Hz",
                meta.sample_rate_hz.unwrap_or(0.0));
            println!("  Canaux       :");
            println!("    suspension_travel : {}", meta.channels.suspension_travel);
            println!("    tyre_slip         : {}", meta.channels.tyre_slip);
            println!("    tyre_load         : {}", meta.channels.tyre_load);
            println!("    tyre_temp         : {}", meta.channels.tyre_temp);
            println!("    position          : {}", meta.channels.position);
        }

        Commands::Replay { path, samples } => {
            let (meta, loaded): (_, Vec<AcSample>) = SessionStore::load(&path)?;

            println!("=== Replay : {} ===", path);
            println!("Sim={} | {} samples | {:.1}s\n",
                meta.simulator, loaded.len(), meta.duration_s);

            for (i, s) in loaded.iter().take(samples).enumerate() {
                println!(
                    "[{i:4}] t={:8}ms  v={:6.1}km/h  ay={:+.3}g  yaw={:+.4}rad/s  steer={:+.4}rad",
                    s.timestamp_ms(),
                    s.speed_ms() * 3.6,
                    s.acceleration_g()[0],
                    s.yaw_rate_rads(),
                    s.steering_angle_rad(),
                );
            }
        }
    }

    Ok(())
}