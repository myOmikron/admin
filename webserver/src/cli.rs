//! Definitions of the CLI arguments

use clap::Parser;
use clap::Subcommand;

/// Entrypoint for the admin server
#[derive(Parser)]
pub struct Cli {
    /// The available subcommands
    #[clap(subcommand)]
    pub command: Command,
}

/// All available commands
#[derive(Subcommand)]
pub enum Command {
    /// Start the server
    Start,
    /// Run the migrations on the database
    Migrate {
        /// The directory where the migration files are located in
        migrations_dir: String,
    },
    /// Create new migrations
    #[cfg(debug_assertions)]
    MakeMigrations {
        /// The directory where the migration files are located in
        migrations_dir: String,
    },
    /// Creates an invitation for an admin user
    CreateAdmin,
}
