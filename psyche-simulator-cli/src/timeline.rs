use psyche::core::sensor::SensorID;
use psyche::core::Scalar;
use serde::{Deserialize, Serialize};
use serde_json::Result as JsonResult;
use serde_yaml::Result as YamlResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub playing_mode: PlayingMode,
    pub actions: Vec<Action>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            playing_mode: Default::default(),
            actions: vec![Action {
                time: 0.0,
                action_type: ActionType::IgniteRandomSynapsesByPercentage(1.0, (1.0, 1.0)),
            }],
        }
    }
}

impl Timeline {
    #[inline]
    pub fn from_json(json: &str) -> JsonResult<Self> {
        serde_json::from_str(json)
    }

    #[inline]
    pub fn from_yaml(yaml: &str) -> YamlResult<Self> {
        serde_yaml::from_str(yaml)
    }

    #[inline]
    pub fn to_json(&self) -> JsonResult<String> {
        serde_json::to_string_pretty(self)
    }

    #[inline]
    pub fn to_yaml(&self) -> YamlResult<String> {
        serde_yaml::to_string(self)
    }

    pub fn perform(&self, mut start: Scalar, mut end: Scalar) -> Option<Vec<Action>> {
        match self.playing_mode {
            PlayingMode::Infinite => {
                if let Some(duration) = self
                    .actions
                    .iter()
                    .map(|a| a.time)
                    .max_by(|a, b| a.partial_cmp(&b).unwrap())
                {
                    if start <= duration {
                        Some(
                            self.actions
                                .iter()
                                .filter(|a| a.time >= start && a.time < end)
                                .cloned()
                                .collect(),
                        )
                    } else {
                        Some(vec![])
                    }
                } else {
                    Some(vec![])
                }
            }
            PlayingMode::Loop => {
                if let Some(duration) = self
                    .actions
                    .iter()
                    .map(|a| a.time)
                    .max_by(|a, b| a.partial_cmp(&b).unwrap())
                {
                    while start > duration {
                        start -= duration;
                    }
                    while end > duration {
                        end -= duration;
                    }
                    Some(
                        self.actions
                            .iter()
                            .filter(|a| a.time >= start && a.time < end)
                            .cloned()
                            .collect(),
                    )
                } else {
                    Some(vec![])
                }
            }
            PlayingMode::Once => {
                if let Some(duration) = self
                    .actions
                    .iter()
                    .map(|a| a.time)
                    .max_by(|a, b| a.partial_cmp(&b).unwrap())
                {
                    if start <= duration {
                        Some(
                            self.actions
                                .iter()
                                .filter(|a| a.time >= start && a.time < end)
                                .cloned()
                                .collect(),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayingMode {
    Infinite,
    Loop,
    Once,
}

impl Default for PlayingMode {
    fn default() -> Self {
        PlayingMode::Once
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub time: Scalar,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    None,
    TriggerSensorByID(SensorID, (Scalar, Scalar)),
    TriggerSensorByIndex(usize, (Scalar, Scalar)),
    TriggerRandomSensorsByPercentage(Scalar, (Scalar, Scalar)),
    TriggerRandomSensorsByAmount(usize, (Scalar, Scalar)),
    IgniteRandomSynapsesByPercentage(Scalar, (Scalar, Scalar)),
    IgniteRandomSynapsesByAmount(usize, (Scalar, Scalar)),
}
