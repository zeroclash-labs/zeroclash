use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Sequence, Value};
use std::collections::HashSet;

/// Represents prepend/append/delete operations on a YAML sequence field (rules, proxies, groups).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeqMap {
    pub prepend: Sequence,
    pub append: Sequence,
    pub delete: Vec<String>,
}

fn collect_proxy_names(seq: &Sequence) -> Vec<String> {
    seq.iter()
        .filter_map(|item| match item {
            Value::Mapping(map) => map.get("name").and_then(Value::as_str).map(str::to_owned),
            Value::String(name) => Some(name.to_owned()),
            _ => None,
        })
        .collect()
}

fn is_selector_group(group_map: &Mapping) -> bool {
    group_map
        .get("type")
        .and_then(Value::as_str)
        .map(|value| {
            let value = value.to_ascii_lowercase();
            value == "select" || value == "selector"
        })
        .unwrap_or(false)
}

/// Apply prepend/append/delete operations to a named field in the clash config.
pub fn use_seq(seq: SeqMap, mut config: Mapping, field: &str) -> Mapping {
    let SeqMap {
        prepend,
        append,
        delete,
    } = seq;

    let added_proxy_names = if field == "proxies" {
        let mut names = collect_proxy_names(&prepend);
        names.extend(collect_proxy_names(&append));
        let mut seen = HashSet::new();
        names
            .into_iter()
            .filter(|name| seen.insert(name.clone()))
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let mut new_seq = Sequence::new();
    new_seq.extend(prepend);

    if let Some(Value::Sequence(origin)) = config.get(field) {
        let filtered: Sequence = origin
            .iter()
            .filter(|item| match item {
                Value::String(s) => !delete.contains(s),
                Value::Mapping(m) => m
                    .get("name")
                    .and_then(Value::as_str)
                    .is_none_or(|name| !delete.iter().any(|d| d.as_str() == name)),
                _ => true,
            })
            .cloned()
            .collect();
        new_seq.extend(filtered);
    }

    new_seq.extend(append);
    config.insert(Value::String(field.into()), Value::Sequence(new_seq));

    // When processing proxies, also update proxy-groups to remove deleted proxies
    if field == "proxies"
        && let Some(Value::Sequence(groups)) = config.get_mut("proxy-groups")
    {
        let mut new_groups = Sequence::new();
        let mut appended_to_selector = false;
        for group in groups {
            if let Value::Mapping(group_map) = group {
                let mut proxies_seq =
                    group_map
                        .get("proxies")
                        .and_then(Value::as_sequence)
                        .map(|proxies| {
                            proxies
                                .iter()
                                .filter(|p| match p {
                                    Value::String(name) => {
                                        !delete.iter().any(|d| d.as_str() == name)
                                    }
                                    _ => true,
                                })
                                .cloned()
                                .collect::<Sequence>()
                        });

                if !appended_to_selector
                    && !added_proxy_names.is_empty()
                    && is_selector_group(group_map)
                {
                    let base_seq = proxies_seq.unwrap_or_else(Sequence::new);
                    let mut seq = Sequence::new();
                    let mut existing = HashSet::new();
                    for name in &added_proxy_names {
                        if existing.insert(name.clone()) {
                            seq.push(Value::String(name.clone()));
                        }
                    }
                    for value in base_seq {
                        if let Value::String(name) = &value
                            && !existing.insert(name.to_owned())
                        {
                            continue;
                        }
                        seq.push(value);
                    }
                    proxies_seq = Some(seq);
                    appended_to_selector = true;
                }

                if let Some(seq) = proxies_seq {
                    group_map.insert(Value::String("proxies".into()), Value::Sequence(seq));
                }
                new_groups.push(Value::Mapping(group_map.to_owned()));
            } else {
                new_groups.push(group.to_owned());
            }
        }
        config.insert(
            Value::String("proxy-groups".into()),
            Value::Sequence(new_groups),
        );
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_proxy_and_references() {
        let config_str = r#"
proxies:
- name: "proxy1"
  type: "ss"
- name: "proxy2"
  type: "vmess"
proxy-groups:
- name: "group1"
  type: "select"
  proxies:
    - "proxy1"
    - "proxy2"
- name: "group2"
  type: "select"
  proxies:
    - "proxy1"
"#;
        let mut config: Mapping =
            serde_yaml_ng::from_str(config_str).expect("Failed to parse test YAML");
        let seq = SeqMap {
            prepend: Sequence::new(),
            append: Sequence::new(),
            delete: vec!["proxy1".to_string()],
        };
        config = use_seq(seq, config, "proxies");

        let proxies = config
            .get("proxies")
            .expect("proxies field")
            .as_sequence()
            .expect("proxies seq");
        assert_eq!(proxies.len(), 1);

        let groups = config
            .get("proxy-groups")
            .expect("proxy-groups field")
            .as_sequence()
            .expect("proxy-groups seq");
        let g1 = groups[0].as_mapping().expect("group mapping");
        let g1p = g1
            .get("proxies")
            .expect("group proxies")
            .as_sequence()
            .unwrap();
        assert_eq!(g1p.len(), 1);
        assert_eq!(g1p[0].as_str().unwrap(), "proxy2");

        let g2 = groups[1].as_mapping().expect("group mapping");
        let g2p = g2
            .get("proxies")
            .expect("group proxies")
            .as_sequence()
            .unwrap();
        assert_eq!(g2p.len(), 0);
    }
}
