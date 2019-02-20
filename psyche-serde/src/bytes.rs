use bincode::Result as BinResult;
use psyche_core::brain::{Brain, BrainActivityMap};
use psyche_core::brain_builder::BrainBuilder;
use psyche_core::config::Config;
use psyche_core::offspring_builder::OffspringBuilder;

#[inline]
pub fn brain_to_bytes(brain: &Brain) -> BinResult<Vec<u8>> {
    bincode::serialize(brain)
}

#[inline]
pub fn brain_from_bytes(bytes: &[u8]) -> BinResult<Brain> {
    bincode::deserialize(bytes)
}

#[inline]
pub fn brain_activity_map_to_bytes(bam: &BrainActivityMap) -> BinResult<Vec<u8>> {
    bincode::serialize(bam)
}

#[inline]
pub fn brain_activity_map_from_bytes(bytes: &[u8]) -> BinResult<BrainActivityMap> {
    bincode::deserialize(bytes)
}

#[inline]
pub fn config_to_bytes(config: &Config) -> BinResult<Vec<u8>> {
    bincode::serialize(config)
}

#[inline]
pub fn config_from_bytes(bytes: &[u8]) -> BinResult<Config> {
    bincode::deserialize(bytes)
}

#[inline]
pub fn brain_builder_to_bytes(brain_builder: &BrainBuilder) -> BinResult<Vec<u8>> {
    bincode::serialize(brain_builder)
}

#[inline]
pub fn brain_builder_from_bytes(bytes: &[u8]) -> BinResult<BrainBuilder> {
    bincode::deserialize(bytes)
}

#[inline]
pub fn offspring_builder_to_bytes(offspring_builder: &OffspringBuilder) -> BinResult<Vec<u8>> {
    bincode::serialize(offspring_builder)
}

#[inline]
pub fn offspring_builder_from_bytes(bytes: &[u8]) -> BinResult<OffspringBuilder> {
    bincode::deserialize(bytes)
}
