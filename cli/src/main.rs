use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "simracing-fit", about = "Identification de modèles véhicule à partir de télémétrie")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enregistre une session depuis Assetto Corsa (UDP)
    RecordAc {
        #[arg(long, default_value = "0.0.0.0:9996")]
        addr: String,
        #[arg(long, default_value_t = 3600)]
        max_samples: usize,
    },
    /// Affiche les infos d'un fichier de session (à venir)
    Inspect {
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("simracing_fit=debug".parse()?)
            .add_directive("telemetry_ac=debug".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::RecordAc { addr, max_samples } => {
            use telemetry_ac::AcUdpReader;
            use telemetry_core::TelemetrySample;

            let reader = AcUdpReader::bind(&addr).await?;
            let session = reader.record_session(max_samples).await?;

            println!("=== Session AC enregistrée ===");
            println!("Simulateur : {}", session.simulator);
            println!("Samples    : {}", session.samples.len());
            println!("Durée      : {:.1}s", session.duration_s());
            if let Some(hz) = session.sample_rate_hz() {
                println!("Fréquence  : {:.0} Hz", hz);
            }
            if let Some(first) = session.samples.first() {
                println!("\n--- Premier sample ---");
                println!("  Vitesse      : {:.1} km/h", first.speed_ms() * 3.6);
                println!("  Acc latérale : {:.3} g", first.acceleration_g()[0]);
                println!("  Acc long.    : {:.3} g", first.acceleration_g()[1]);
                println!("  Lacet        : {:.4} rad/s", first.yaw_rate_rads());
                println!("  Braquage     : {:.4} rad", first.steering_angle_rad());
            }
        }
        Commands::Inspect { path } => {
            println!("Inspect '{path}' — pas encore implémenté");
        }
    }

    Ok(())
}