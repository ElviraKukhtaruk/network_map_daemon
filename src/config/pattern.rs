use std::path::PathBuf;
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::io::Write;


#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Set custom path to the configuration file [./Config.toml default]
    #[arg(short, long, value_name = "Path to the config file")]
    pub config: Option<PathBuf>,

    #[arg(short, long, value_name = "Clickhouse hostname")]
    pub servername: Option<String>,

    /// [9000 default]
    #[arg(short, long, value_name = "Clickhouse native port")]
    pub native_port: Option<u16>,

    #[arg(short, long, value_name = "Clickhouse password")]
    pub password: Option<String>,

    #[arg(short, long, value_name = "Clickhouse database name")]
    pub db: Option<String>,

    #[arg(short, long, value_name = "Clickhouse user")]
    pub user: Option<PathBuf>,
}


#[derive(Serialize)]
pub struct ServerConfigStructure {
    pub user: String,
    pub password: String,
    pub db: String,
    pub hostname: String,
    pub native_port: u16,
}
