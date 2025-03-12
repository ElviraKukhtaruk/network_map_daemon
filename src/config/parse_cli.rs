use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Set custom path to the configuration file [./Config.toml default]
    #[arg(short, long, value_name = "Path to the config file")]
    pub config: Option<PathBuf>,

    #[arg(short, long, value_name = "Clickhouse hostname")]
    pub servername: Option<String>,

    /// [9000 default]
    #[arg(long, value_name = "Clickhouse native port")]
    pub port: Option<u16>,

    #[arg(short, long, value_name = "Clickhouse password")]
    pub password: Option<String>,

    #[arg(short, long, value_name = "Clickhouse database name")]
    pub db: Option<String>,

    #[arg(short, long, value_name = "Clickhouse user")]
    pub user: Option<String>,

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
    pub lat: Option<f32>,

    #[arg(long, value_name = "Server lng")]
    pub lng: Option<f32>,

    #[arg(long, value_name = "Server priority")]
    pub priority: Option<u8>,

    /// Set world's map point of view.
    /// If `true`, the 3D world map initializes with the specified server at the center,
    /// adjusting rotation and position accordingly.
    #[arg(long, value_name = "Map center")]
    pub center: Option<bool>,

    /// Specifies the directory path for saving log files.
    #[arg(long, value_name = "Path", default_value_t = String::from("./stats_logs/logs.log"))]
    pub logs_path: String
}
