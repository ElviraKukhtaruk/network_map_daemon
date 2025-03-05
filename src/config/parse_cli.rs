use std::path::PathBuf;
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::fmt;


#[derive(Parser, Debug)]
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

    /// [/etc/machine-id default, generate random if not provided]
    #[arg(long, value_name = "Server ID")]
    pub server_id: Option<String>,

    /// [/etc/hostname default]
    #[arg(long, value_name = "Server hostname")]
    pub hostname: Option<String>,

    #[arg(long, value_name = "Server label")]
    pub label: Option<String>,

    /// Scan defined interface names (Regex supported): --interface_filter eth0, eth1
    #[arg(long, value_delimiter = ',', value_name = "Server interface filter")]
    pub interface_filter: Vec<Option<String>>,

    #[arg(long, value_name = "Server country location")]
    pub country: Option<String>,

    #[arg(long, value_name = "Server city location")]
    pub city: Option<String>,

    #[arg(long, value_name = "Server lat")]
    pub lat: Option<f64>,

    #[arg(long, value_name = "Server lng")]
    pub lng: Option<f64>,

}

#[derive(Serialize)]
pub struct ServerConfigStructure {
    pub user: String,
    pub password: String,
    pub db: String,
    pub hostname: String,
    pub native_port: u16,
}
