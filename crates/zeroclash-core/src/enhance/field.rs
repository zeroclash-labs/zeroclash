use serde_yaml_ng::{Mapping, Value};
use std::collections::HashSet;

/// Fields that appear first in sorted output (handle/config fields).
pub const HANDLE_FIELDS: [&str; 12] = [
    "mode",
    "redir-port",
    "tproxy-port",
    "mixed-port",
    "socks-port",
    "port",
    "allow-lan",
    "log-level",
    "ipv6",
    "external-controller",
    "secret",
    "unified-delay",
];

/// Fields that appear last in sorted output (proxy/rule collections).
pub const DEFAULT_FIELDS: [&str; 5] = [
    "proxies",
    "proxy-providers",
    "proxy-groups",
    "rule-providers",
    "rules",
];

/// Convert all keys in a YAML mapping to lowercase.
pub fn use_lowercase(config: &Mapping) -> Mapping {
    let mut ret = Mapping::new();
    for (key, value) in config {
        if let Some(key_str) = key.as_str() {
            let lower = key_str.to_ascii_lowercase();
            ret.insert(Value::from(lower), value.clone());
        }
    }
    ret
}

/// Sort a YAML mapping: handle fields first, then user fields, then proxy/rule collections.
pub fn use_sort(config: Mapping) -> Mapping {
    let mut ret = Mapping::new();

    // Handle fields first
    for &key in &HANDLE_FIELDS {
        let k = Value::from(key);
        if let Some(v) = config.get(&k) {
            ret.insert(k, v.clone());
        }
    }

    // User-defined fields (not handle, not default)
    let supported: HashSet<&str> = HANDLE_FIELDS
        .iter()
        .chain(DEFAULT_FIELDS.iter())
        .copied()
        .collect();
    for (key, value) in &config {
        if let Some(k) = key.as_str()
            && !supported.contains(k)
        {
            ret.insert(key.clone(), value.clone());
        }
    }

    // Default fields last
    for &key in &DEFAULT_FIELDS {
        let k = Value::from(key);
        if let Some(v) = config.get(&k) {
            ret.insert(k, v.clone());
        }
    }

    ret
}

/// Collect all keys from a mapping, lowercased. Useful for tracking which keys were modified.
pub fn use_keys<'a>(config: &'a Mapping) -> impl Iterator<Item = String> + 'a {
    config
        .iter()
        .filter_map(|(key, _)| key.as_str())
        .map(|s| s.to_ascii_lowercase())
}
