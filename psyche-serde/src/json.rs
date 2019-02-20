use psyche_core::brain::{Brain, BrainActivityMap};
use psyche_core::brain_builder::BrainBuilder;
use psyche_core::config::Config;
use psyche_core::offspring_builder::OffspringBuilder;
use serde_json::Result as JsonResult;

#[inline]
pub fn brain_to_json(brain: &Brain, pretty: bool) -> JsonResult<String> {
    if pretty {
        serde_json::to_string_pretty(brain)
    } else {
        serde_json::to_string(brain)
    }
}

#[inline]
pub fn brain_from_json(json: &str) -> JsonResult<Brain> {
    serde_json::from_str(json)
}

#[inline]
pub fn brain_activity_map_to_json(bam: &BrainActivityMap, pretty: bool) -> JsonResult<String> {
    if pretty {
        serde_json::to_string_pretty(bam)
    } else {
        serde_json::to_string(bam)
    }
}

#[inline]
pub fn brain_activity_map_from_json(json: &str) -> JsonResult<BrainActivityMap> {
    serde_json::from_str(json)
}

#[inline]
pub fn config_to_json(config: &Config, pretty: bool) -> JsonResult<String> {
    if pretty {
        serde_json::to_string_pretty(config)
    } else {
        serde_json::to_string(config)
    }
}

#[inline]
pub fn config_from_json(json: &str) -> JsonResult<Config> {
    serde_json::from_str(json)
}

#[inline]
pub fn brain_builder_to_json(brain_builder: &BrainBuilder, pretty: bool) -> JsonResult<String> {
    if pretty {
        serde_json::to_string_pretty(brain_builder)
    } else {
        serde_json::to_string(brain_builder)
    }
}

#[inline]
pub fn brain_builder_from_json(json: &str) -> JsonResult<BrainBuilder> {
    serde_json::from_str(json)
}

#[inline]
pub fn offspring_builder_to_json(
    offspring_builder: &OffspringBuilder,
    pretty: bool,
) -> JsonResult<String> {
    if pretty {
        serde_json::to_string_pretty(offspring_builder)
    } else {
        serde_json::to_string(offspring_builder)
    }
}

#[inline]
pub fn offspring_builder_from_json(json: &str) -> JsonResult<OffspringBuilder> {
    serde_json::from_str(json)
}
