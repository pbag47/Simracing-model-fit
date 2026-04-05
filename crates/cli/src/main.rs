use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use telemetry::{GenericSample, SimulatorKind, TelemetrySample};
use session_store::{SessionStore, format::ChannelManifest};
use identification::{SampleFilter, FilterCriteria};


#[derive(Parser)]
#[command(name = "simracing-fit", about = "Identification de modèles véhicule à partir de télémétrie")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enregistre une session UDP dans un fichier .srf
    Record {
        #[arg(long, default_value = "ac")]
        simulator: SimulatorKind,
        #[arg(long, default_value = "127.0.0.1:9997")]
        local_addr: String,
        #[arg(long, default_value = "127.0.0.1:9996")]
        udp_server_addr: String,
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
    /// Analyse les samples valides pour l'identification bicyclette
    FilterStats {
        path: String,
        #[arg(long, default_value = "default")]
        criteria: String, // "default" ou "relaxed"
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
        Commands::Record { simulator, local_addr, udp_server_addr, max_samples, output } => {
            let session = telemetry::record(simulator, &local_addr, &udp_server_addr, max_samples).await?;

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
            println!("  Fréquence    : {:.0} Hz",meta.sample_rate_hz.unwrap_or(0.0));
            println!("  Canaux       :");
            println!("    suspension_travel : {}", meta.channels.suspension_travel);
            println!("    tyre_slip         : {}", meta.channels.tyre_slip);
            println!("    tyre_load         : {}", meta.channels.tyre_load);
            println!("    tyre_temp         : {}", meta.channels.tyre_temp);
            println!("    position          : {}", meta.channels.position);
        }

        Commands::Replay { path, samples } => {
            let (meta, loaded): (_, Vec<GenericSample>) = SessionStore::load(&path)?;

            println!("=== Replay : {} ===", path);
            println!("Sim={} | {} samples | {:.1}s\n",
                meta.simulator, loaded.len(), meta.duration_s);

            for (i, s) in loaded.iter().take(samples).enumerate() {
                println!(
                    "[{i:4}] t={:8}ms  v={:6.1}km/h  ay={:+.3}g  steer_angle={:+.4}rad  throttle={:.2}%",
                    s.timestamp_ms(),
                    s.speed_ms() * 3.6,
                    s.acceleration_g()[0],
                    s.steering_angle_rad(),
                    s.throttle_norm()
                );
            }
        }

        Commands::FilterStats { path, criteria } => {
            let (_, samples): (_, Vec<GenericSample>) = SessionStore::load(&path)?;
            let crit = if criteria == "relaxed" {
                FilterCriteria::relaxed()
            } else {
                FilterCriteria::default()
            };

            let (accepted, stats) = SampleFilter::filter(&samples, &crit);
            stats.print_summary();
            println!("\nSamples utilisables pour identification : {}", accepted.len());
        }

    }

    Ok(())
}