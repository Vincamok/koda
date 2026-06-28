use std::process::Command;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "koda", about = "Koda workspace CLI", version)]
struct Cli {
    /// Koda API base URL (overrides KODA_API_URL env)
    #[arg(long, env = "KODA_API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Open an SSH session into a workspace
    Connect {
        /// Workspace UID (UUID)
        uid: String,

        /// SSH user inside the container (default: root)
        #[arg(long, default_value = "root")]
        user: String,

        /// SSH port hint (resolved via API when omitted)
        #[arg(long)]
        port: Option<u16>,

        /// SSH hostname (resolved via API when omitted, default: koda host)
        #[arg(long)]
        host: Option<String>,
    },

    /// List workspaces for an organization
    List {
        /// Organization ID
        #[arg(long, env = "KODA_ORG_ID")]
        org: String,
    },
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct WorkspaceSsh {
    ssh_host: String,
    ssh_port: u16,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Connect { uid, user, port, host } => {
            let (ssh_host, ssh_port) = resolve_ssh(&cli.api_url, &uid, host, port)?;

            println!("Connecting to workspace {uid} via SSH ({ssh_host}:{ssh_port})…");

            let status = Command::new("ssh")
                .args([
                    "-o", "StrictHostKeyChecking=no",
                    "-o", "UserKnownHostsFile=/dev/null",
                    "-p", &ssh_port.to_string(),
                    &format!("{user}@{ssh_host}"),
                ])
                .status()
                .context("failed to execute ssh")?;

            std::process::exit(status.code().unwrap_or(1));
        }

        Cmd::List { org } => {
            let client = reqwest::blocking::Client::builder()
                .cookie_store(true)
                .build()?;

            let url = format!("{}/api/v1/organizations/{}/workspaces", cli.api_url, org);
            let resp = client
                .get(&url)
                .header("Accept", "application/json")
                .send()
                .context("failed to contact Koda API")?;

            if !resp.status().is_success() {
                bail!("API returned {}: {}", resp.status(), resp.text().unwrap_or_default());
            }

            #[derive(Deserialize)]
            struct Workspace {
                uid: String,
                name: String,
                status: String,
            }

            #[derive(Deserialize)]
            struct Page {
                data: Vec<Workspace>,
            }

            let page: ApiResponse<Page> = resp.json()?;
            println!("{:<38} {:<20} {}", "UID", "NAME", "STATUS");
            for ws in &page.data.data {
                println!("{:<38} {:<20} {}", ws.uid, ws.name, ws.status);
            }
        }
    }

    Ok(())
}

/// Resolve SSH host and port for a workspace.
/// Calls GET /api/v1/workspaces/:uid/ssh — the API returns the sozu TCP route.
/// Falls back to command-line overrides if provided.
fn resolve_ssh(
    api_url: &str,
    uid: &str,
    host_override: Option<String>,
    port_override: Option<u16>,
) -> anyhow::Result<(String, u16)> {
    if let (Some(h), Some(p)) = (host_override.clone(), port_override) {
        return Ok((h, p));
    }

    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()?;

    let url = format!("{}/api/v1/workspaces/{}/ssh", api_url, uid);
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .context("failed to contact Koda API")?;

    if resp.status().is_success() {
        let data: ApiResponse<WorkspaceSsh> = resp.json()?;
        let h = host_override.unwrap_or(data.data.ssh_host);
        let p = port_override.unwrap_or(data.data.ssh_port);
        return Ok((h, p));
    }

    // Fallback: derive from KODA_API_URL host
    let host = host_override.unwrap_or_else(|| {
        url::Url::parse(api_url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_else(|| "localhost".into())
    });
    let port = port_override.unwrap_or(2200);

    Ok((host, port))
}
