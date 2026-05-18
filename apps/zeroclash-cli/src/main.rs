//! ZeroClash CLI — headless operation for scripting and automation.
//!
//! Subcommands: core, config, profile, log

use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::PathBuf;

use zeroclash_core::mihomo::MihomoClient;
use zeroclash_core::ProfileStore;

/// ZeroClash CLI — manage the mihomo proxy core from the command line.
#[derive(Parser)]
#[command(name = "zeroclash", version, about)]
struct Cli {
    /// Mihomo external controller address (default: 127.0.0.1:9097)
    #[arg(short, long, default_value = "http://127.0.0.1:9097")]
    controller: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Core lifecycle operations
    Core {
        #[command(subcommand)]
        action: CoreAction,
    },
    /// Configuration operations
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Profile / subscription management
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Log and monitoring
    Log {
        #[command(subcommand)]
        action: LogAction,
    },
}

// ── Core ────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum CoreAction {
    /// Get core version and running status
    Status,
    /// Switch proxy mode (rule/global/direct)
    Mode { mode: String },
    /// Get current config
    Config,
    /// Get active connections count
    Connections,
    /// Get traffic stats (up/down bytes)
    Traffic,
    /// Close a connection by ID
    Close { id: String },
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current config as JSON
    Show,
    /// Get a specific config value
    Get { key: String },
    /// Patch config with a JSON value
    Set { key: String, value: String },
}

// ── Profile ─────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ProfileAction {
    /// List all profiles
    List,
    /// Import a remote profile from URL
    Import {
        /// Subscription URL
        url: String,
        /// Profile name (optional)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Set the active profile
    Switch { uid: String },
    /// Delete a profile
    Delete { uid: String },
    /// Show the current profile's YAML content
    Show,
}

// ── Log ─────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum LogAction {
    /// Show recent proxy selection events
    Proxies,
    /// Show current rules
    Rules,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = MihomoClient::new(&cli.controller);

    match cli.command {
        Commands::Core { action } => handle_core(&client, action).await,
        Commands::Config { action } => handle_config(&client, action).await,
        Commands::Profile { action } => handle_profile(action).await,
        Commands::Log { action } => handle_log(&client, action).await,
    }
}

// ── Core handlers ──────────────────────────────────────────────────────────

async fn handle_core(client: &MihomoClient, action: CoreAction) -> anyhow::Result<()> {
    match action {
        CoreAction::Status => {
            match client.version().await {
                Ok(v) => println!("Core running — version: {v}"),
                Err(_) => println!("Core not reachable"),
            }
        }
        CoreAction::Mode { mode } => {
            client.switch_mode(&mode).await?;
            println!("Mode switched to: {mode}");
        }
        CoreAction::Config => {
            let cfg = client.configs().await?;
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        CoreAction::Connections => {
            let conns = client.connections().await?;
            let count = conns
                .get("connections")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            println!("Active connections: {count}");
            println!("{}", serde_json::to_string_pretty(&conns)?);
        }
        CoreAction::Traffic => {
            let t = client.traffic().await?;
            println!("Upload:   {}", format_bytes(t.up));
            println!("Download: {}", format_bytes(t.down));
        }
        CoreAction::Close { id } => {
            client.close_connection(&id).await?;
            println!("Connection {id} closed");
        }
    }
    Ok(())
}

// ── Config handlers ────────────────────────────────────────────────────────

async fn handle_config(client: &MihomoClient, action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Show => {
            let cfg = client.configs().await?;
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        ConfigAction::Get { key } => {
            let cfg = client.configs().await?;
            match cfg.get(&key) {
                Some(v) => println!("{}", serde_json::to_string_pretty(v)?),
                None => anyhow::bail!("key '{key}' not found in config"),
            }
        }
        ConfigAction::Set { key, value } => {
            let val: Value = serde_json::from_str(&value)?;
            let patch = serde_json::json!({ &key: val });
            client.patch_config(&patch).await?;
            println!("Config patched: {key} = {value}");
        }
    }
    Ok(())
}

// ── Profile handlers ────────────────────────────────────────────────────────

async fn handle_profile(action: ProfileAction) -> anyhow::Result<()> {
    let data_dir = dirs_next()
        .join("zeroclash");
    std::fs::create_dir_all(&data_dir)?;

    match action {
        ProfileAction::List => {
            let store = ProfileStore::load(data_dir).await?;
            for p in store.preview() {
                let active = if p.is_current { " *" } else { "  " };
                println!("{active} [{:6}] {:24} {}", p.itype, p.name, p.uid);
            }
        }
        ProfileAction::Import { url, name } => {
            let store = ProfileStore::load(data_dir.clone()).await?;
            let item = store.fetch_remote(&url, name.as_deref(), None).await?;
            let mut store = ProfileStore::load(data_dir).await?;
            let _ = store.add_item(item);
            store.save().await?;
            println!("Profile imported successfully");
        }
        ProfileAction::Switch { uid } => {
            let mut store = ProfileStore::load(data_dir).await?;
            store.set_current(&uid)?;
            store.save().await?;
            println!("Switched to profile: {uid}");
        }
        ProfileAction::Delete { uid } => {
            let mut store = ProfileStore::load(data_dir).await?;
            store.delete_item(&uid)?;
            store.save().await?;
            println!("Profile deleted: {uid}");
        }
        ProfileAction::Show => {
            let store = ProfileStore::load(data_dir).await?;
            let mapping = store.current_mapping().await?;
            println!("{}", serde_yaml_ng::to_string(&mapping)?);
        }
    }
    Ok(())
}

// ── Log handlers ────────────────────────────────────────────────────────────

async fn handle_log(client: &MihomoClient, action: LogAction) -> anyhow::Result<()> {
    match action {
        LogAction::Proxies => {
            let proxies = client.proxies().await?;
            println!("{}", serde_json::to_string_pretty(&proxies)?);
        }
        LogAction::Rules => {
            let rules = client.rules().await?;
            println!("{}", serde_json::to_string_pretty(&rules)?);
        }
    }
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn dirs_next() -> PathBuf {
    dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
