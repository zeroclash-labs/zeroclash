//! macOS system proxy management via `networksetup` CLI.

use anyhow::{Context as _, Result};
use std::process::Command;

/// Enable system HTTP/HTTPS/SOCKS proxy pointing to `host:port`.
pub fn enable(host: &str, port: u16, bypass: &str) -> Result<()> {
    let services = active_network_services()?;
    for service in &services {
        set_proxy_state(service, "webproxy", host, port, true)?;
        set_proxy_state(service, "securewebproxy", host, port, true)?;
        set_proxy_state(service, "socksfirewallproxy", host, port, true)?;
    }
    if !services.is_empty() {
        set_bypass(&services[0], bypass)?;
    }
    Ok(())
}

/// Disable system proxy on all active network services.
pub fn disable() -> Result<()> {
    let services = active_network_services()?;
    for service in &services {
        set_proxy_state(service, "webproxy", "", 0, false)?;
        set_proxy_state(service, "securewebproxy", "", 0, false)?;
        set_proxy_state(service, "socksfirewallproxy", "", 0, false)?;
    }
    Ok(())
}

fn set_proxy_state(
    service: &str,
    proxy_type: &str,
    host: &str,
    port: u16,
    enable: bool,
) -> Result<()> {
    if enable {
        let status = Command::new("networksetup")
            .args([
                &format!("-set{proxy_type}"),
                service,
                host,
                &port.to_string(),
            ])
            .status()
            .context("failed to set proxy")?;
        if !status.success() {
            log::warn!("networksetup -set{proxy_type} failed for {service}");
        }
    }
    let state = if enable { "on" } else { "off" };
    Command::new("networksetup")
        .args([&format!("-set{proxy_type}state"), service, state])
        .status()
        .context("failed to set proxy state")?;
    Ok(())
}

fn set_bypass(service: &str, bypass: &str) -> Result<()> {
    let domains: Vec<&str> = bypass.split(',').filter(|s| !s.is_empty()).collect();
    let mut args = vec!["-setproxybypassdomains", service];
    args.extend(&domains);
    Command::new("networksetup")
        .args(args)
        .status()
        .context("failed to set bypass domains")?;
    Ok(())
}

/// Get the list of active network services.
fn active_network_services() -> Result<Vec<String>> {
    let output = Command::new("networksetup")
        .args(["-listallnetworkservices"])
        .output()
        .context("failed to list network services")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // First line is "An asterisk (*) denotes that a network service is disabled."
    // Skip it and filter out disabled services (prefixed with *)
    Ok(stdout
        .lines()
        .skip(1)
        .filter(|s| !s.trim().is_empty() && !s.trim().starts_with('*'))
        .map(|s| s.trim().to_string())
        .collect())
}
