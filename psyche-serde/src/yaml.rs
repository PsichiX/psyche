use psyche_core::brain::{Brain, BrainActivityMap};
use psyche_core::brain_builder::BrainBuilder;
use psyche_core::config::Config;
use psyche_core::offspring_builder::OffspringBuilder;
use serde_yaml::Result as YamlResult;

#[inline]
pub fn brain_to_yaml(brain: &Brain) -> YamlResult<String> {
    serde_yaml::to_string(brain)
}

#[inline]
pub fn brain_from_yaml(yaml: &str) -> YamlResult<Brain> {
    serde_yaml::from_str(yaml)
}

#[inline]
pub fn brain_activity_map_to_yaml(bam: &BrainActivityMap) -> YamlResult<String> {
    serde_yaml::to_string(bam)
}

#[inline]
pub fn brain_activity_map_from_yaml(yaml: &str) -> YamlResult<BrainActivityMap> {
    serde_yaml::from_str(yaml)
}

#[inline]
pub fn config_to_yaml(config: &Config) -> YamlResult<String> {
    serde_yaml::to_string(config)
}

#[inline]
pub fn config_from_yaml(yaml: &str) -> YamlResult<Config> {
    serde_yaml::from_str(yaml)
}

#[inline]
pub fn brain_builder_to_yaml(brain_builder: &BrainBuilder) -> YamlResult<String> {
    serde_yaml::to_string(brain_builder)
}

#[inline]
pub fn brain_builder_from_yaml(yaml: &str) -> YamlResult<BrainBuilder> {
    serde_yaml::from_str(yaml)
}

#[inline]
pub fn offspring_builder_to_yaml(offspring_builder: &OffspringBuilder) -> YamlResult<String> {
    serde_yaml::to_string(offspring_builder)
}

#[inline]
pub fn offspring_builder_from_yaml(yaml: &str) -> YamlResult<OffspringBuilder> {
    serde_yaml::from_str(yaml)
}
