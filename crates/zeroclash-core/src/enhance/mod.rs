//! Profile enhancement pipeline — processes clash configs through merge, sequence ops,
//! chain items, TUN configuration, and JavaScript script execution.

mod chain;
mod field;
mod merge;
pub mod script;
pub mod seq;
mod tun;

pub use chain::{ChainItem, ChainType};
pub use field::{use_keys, use_lowercase, use_sort};
pub use merge::use_merge;
pub use script::use_script;
pub use seq::{SeqMap, use_seq};
pub use tun::use_tun;

use crate::config::VergeConfig;
use anyhow::Result;
use serde_yaml_ng::Mapping;

/// Run the full enhancement pipeline on a profile config.
///
/// Pipeline order:
///   1. Lowercase all keys
///   2. Apply Merge chain items
///   3. Apply Script chain items (via Boa JS engine)
///   4. Apply Rules/Proxies/Groups sequence operations
///   5. Apply TUN configuration from VergeConfig
///   6. Sort fields into canonical order
pub async fn enhance(
    mut config: Mapping,
    chain_items: &[ChainItem],
    verge: &VergeConfig,
    profile_name: &str,
) -> Result<Mapping> {
    config = use_lowercase(&config);

    for item in chain_items {
        if let ChainType::Merge(ref merge_mapping) = item.data {
            config = use_merge(merge_mapping, config);
        }
    }

    for item in chain_items {
        if let ChainType::Script(ref script) = item.data {
            let (new_config, _logs) =
                use_script(script.clone(), config, profile_name.to_string()).await?;
            config = new_config;
        }
    }

    for item in chain_items {
        match &item.data {
            ChainType::Rules(seq_map) => {
                config = use_seq(seq_map.clone(), config, "rules");
            }
            ChainType::Proxies(seq_map) => {
                config = use_seq(seq_map.clone(), config, "proxies");
            }
            ChainType::Groups(seq_map) => {
                config = use_seq(seq_map.clone(), config, "proxy-groups");
            }
            _ => {}
        }
    }

    config = use_tun(config, verge.enable_tun);
    config = use_sort(config);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VergeConfig;
    use serde_yaml_ng::{Mapping, Value};

    #[tokio::test]
    async fn test_enhance_empty_chain() {
        let config = Mapping::new();
        let chain_items = vec![];
        let verge = VergeConfig::default();
        let result = enhance(config, &chain_items, &verge, "test").await.unwrap();
        assert!(!result.is_empty()); // TUN section added even for empty config
    }

    #[tokio::test]
    async fn test_enhance_merge_and_tun() {
        let mut config = Mapping::new();
        config.insert("port".into(), Value::Number(7890.into()));

        let mut merge_data = Mapping::new();
        merge_data.insert("mode".into(), Value::String("rule".into()));
        let chain_items = vec![ChainItem {
            uid: "test-merge".into(),
            data: ChainType::Merge(merge_data),
        }];

        let mut verge = VergeConfig::default();
        verge.enable_tun = true;

        let result = enhance(config, &chain_items, &verge, "test").await.unwrap();

        assert_eq!(result.get("mode").unwrap().as_str().unwrap(), "rule");
        assert_eq!(result.get("port").unwrap().as_i64().unwrap(), 7890);
        assert!(result.get("tun").is_some());
    }
}
