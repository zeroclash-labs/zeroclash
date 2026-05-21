use serde_json::Value;
use zeroclash_core::ConnEntry;
use zeroclash_core::mihomo::{DelayHistory, ProxyGroup};

pub fn parse_connections(v: &Value) -> Vec<ConnEntry> {
    let mut entries = Vec::new();
    let conns = match v.get("connections").and_then(|c| c.as_array()) {
        Some(c) => c,
        None => return entries,
    };
    for conn in conns {
        let m = conn.get("metadata");
        entries.push(ConnEntry {
            id: conn
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            host: m
                .and_then(|m| m.get("host"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            network: m
                .and_then(|m| m.get("network"))
                .and_then(|v| v.as_str())
                .unwrap_or("tcp")
                .to_string(),
            conn_type: m
                .and_then(|m| m.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_ip: m
                .and_then(|m| m.get("sourceIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_ip: m
                .and_then(|m| m.get("destinationIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_port: m
                .and_then(|m| m.get("sourcePort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_port: m
                .and_then(|m| m.get("destinationPort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            dns_mode: m
                .and_then(|m| m.get("dnsMode"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            chains: conn
                .get("chains")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            rule: conn
                .get("rule")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            rule_payload: conn
                .get("rulePayload")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            upload: conn.get("upload").and_then(|v| v.as_u64()).unwrap_or(0),
            download: conn.get("download").and_then(|v| v.as_u64()).unwrap_or(0),
            start: conn
                .get("start")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            speed_up: 0,
            speed_down: 0,
        });
    }
    entries
}

pub fn parse_proxy_groups(v: &Value) -> Vec<ProxyGroup> {
    let mut groups = Vec::new();
    let proxies = match v.get("proxies") {
        Some(p) => p,
        None => return groups,
    };
    if let Some(obj) = proxies.as_object() {
        for (name, val) in obj {
            if let Some(typ) = val.get("type").and_then(|t| t.as_str()) {
                let all: Vec<String> = val
                    .get("all")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let now = val.get("now").and_then(|n| n.as_str()).map(String::from);
                let history: Vec<DelayHistory> = val
                    .get("history")
                    .and_then(|h| h.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|v| DelayHistory {
                                time: String::new(),
                                delay: v.get("delay").and_then(|d| d.as_u64()).unwrap_or(0),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                groups.push(ProxyGroup {
                    name: name.clone(),
                    group_type: typ.to_string(),
                    now,
                    all,
                    history,
                });
            }
        }
    }
    groups
}
