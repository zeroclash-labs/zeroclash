use serde_yaml_ng::{Mapping, Value};

/// Apply TUN configuration to the clash config.
///
/// When `enable` is true, sets up DNS settings for TUN mode.
/// When false, only writes `tun.enable = false`.
pub fn use_tun(mut config: Mapping, enable: bool) -> Mapping {
    let tun_key = Value::from("tun");
    let tun_val = config.get(&tun_key);
    let mut tun_val = tun_val.map_or_else(Mapping::new, |val| {
        val.as_mapping().cloned().unwrap_or_else(Mapping::new)
    });

    if enable {
        let dns_key = Value::from("dns");
        let dns_val = config.get(&dns_key);
        let mut dns_val = dns_val.map_or_else(Mapping::new, |val| {
            val.as_mapping().cloned().unwrap_or_else(Mapping::new)
        });
        let ipv6_key = Value::from("ipv6");
        let ipv6_val = config
            .get(&ipv6_key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let current_mode = dns_val
            .get(Value::from("enhanced-mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("fake-ip");

        if current_mode == "fake-ip" || !dns_val.contains_key(Value::from("enhanced-mode")) {
            dns_val.insert(Value::from("enable"), Value::from(true));
            dns_val.insert(Value::from("ipv6"), Value::from(ipv6_val));

            if !dns_val.contains_key(Value::from("enhanced-mode")) {
                dns_val.insert(Value::from("enhanced-mode"), Value::from("fake-ip"));
            }
            if !dns_val.contains_key(Value::from("fake-ip-range")) {
                dns_val.insert(Value::from("fake-ip-range"), Value::from("198.18.0.1/16"));
            }
        }

        config.insert(Value::from("dns"), Value::from(dns_val));
    }

    tun_val.insert(Value::from("enable"), Value::from(enable));
    config.insert(Value::from("tun"), Value::from(tun_val));

    config
}
