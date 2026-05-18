use super::SeqMap;
use serde_yaml_ng::Mapping;

/// A chain item represents a single enhancement operation (merge, script, rule seq, etc.).
#[derive(Debug, Clone)]
pub struct ChainItem {
    pub uid: String,
    pub data: ChainType,
}

/// The type of data a chain item carries.
#[derive(Debug, Clone)]
pub enum ChainType {
    /// Deep-merge overlay YAML
    Merge(Mapping),
    /// JavaScript source to execute (deferred — requires Boa engine)
    Script(String),
    /// Prepend/append/delete on rules
    Rules(SeqMap),
    /// Prepend/append/delete on proxies
    Proxies(SeqMap),
    /// Prepend/append/delete on proxy-groups
    Groups(SeqMap),
}
