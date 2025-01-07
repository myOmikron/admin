//! Utilities to manage the entire IT

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    reason = "Because others (and yourself) will thank you"
)]
#![warn(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "You may add an #[allow] if you comment why your unwrap / expect is necessary"
)]

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::net::SocketAddr;

use clap::Parser;
use galvyn::core::GalvynRouter;
use galvyn::core::Module;
use galvyn::Galvyn;
use rorm::cli as rorm_cli;
use rorm::config::DatabaseConfig;
use rorm::Database;
use rorm::DatabaseDriver;
use tracing::instrument;
use tracing::Level;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

use crate::cli::Cli;
use crate::cli::Command;

pub mod cli;
pub mod http;

/// Creates an invitation for an admin user
async fn create_admin_user() -> Result<(), Box<dyn std::error::Error>> {
    Galvyn::new()
        .register_module::<Database>()
        .init_modules()
        .await?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut mail = String::new();
    let mut display_name = String::new();

    print!("Enter a mail: ");
    stdout.flush()?;
    stdin.read_line(&mut mail)?;
    let mail = mail.trim();

    print!("Enter a display name: ");
    stdout.flush()?;
    stdin.read_line(&mut display_name)?;
    let display_name = display_name.trim().to_string();

    let mut tx = Database::global().start_transaction().await?;

    todo!();

    tx.commit().await?;

    Ok(())
}

#[instrument(skip_all)]
async fn start() -> Result<(), Box<dyn std::error::Error>> {
    Ok(Galvyn::new()
        .register_module::<Database>()
        .init_modules()
        .await?
        .add_routes(GalvynRouter::new())
        .start(SocketAddr::from(([127, 0, 0, 1], 8080)))
        .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_ansi(false)
                .with_filter(filter_fn(|metadata| {
                    if metadata.is_span() {
                        true
                    } else if metadata.is_event() {
                        metadata.target().starts_with("webserver")
                            && *metadata.level() <= Level::WARN
                    } else {
                        false
                    }
                })),
        );

    registry.init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start => start().await?,
        #[cfg(debug_assertions)]
        Command::MakeMigrations { migrations_dir } => {
            use std::io::Write;

            /// Temporary file to store models in
            const MODELS: &str = "/tmp/.models.json";

            let mut file = fs::File::create(MODELS)?;
            rorm::write_models(&mut file)?;
            file.flush()?;

            rorm_cli::make_migrations::run_make_migrations(
                rorm_cli::make_migrations::MakeMigrationsOptions {
                    models_file: MODELS.to_string(),
                    migration_dir: migrations_dir,
                    name: None,
                    non_interactive: false,
                    warnings_disabled: false,
                },
            )?;

            fs::remove_file(MODELS)?;
        }
        Command::Migrate { migrations_dir } => {
            Galvyn::new()
                .register_module::<Database>()
                .init_modules()
                .await?;
            rorm::cli::migrate::run_migrate_custom(
                DatabaseConfig {
                    driver: DatabaseDriver::Postgres {
                        name: env::var("POSTGRES_NAME")?,
                        host: env::var("POSTGRES_HOST")?,
                        port: env::var("POSTGRES_PORT")?.parse()?,
                        user: env::var("POSTGRES_USER")?,
                        password: env::var("POSTGRES_PASSWORD")?,
                    },
                    last_migration_table_name: None,
                },
                migrations_dir,
                false,
                None,
            )
            .await?;
        }
        Command::CreateAdmin => create_admin_user().await?,
    }

    Ok(())
}
