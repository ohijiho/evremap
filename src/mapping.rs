use anyhow::Context;
pub use evdev_rs::enums::{EventCode, EventType, EV_KEY as KeyCode};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MappingConfig {
    pub device_name: String,
    pub phys: Option<String>,
    pub mappings: Vec<Mapping>,
}

impl MappingConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let toml_data = std::fs::read_to_string(path)
            .context(format!("reading toml from {}", path.display()))?;
        let config_file: ConfigFile =
            toml::from_str(&toml_data).context(format!("parsing toml from {}", path.display()))?;
        let mut mappings = vec![];
        for x in config_file.remap {
            mappings.push(x.into());
        }
        Ok(Self {
            device_name: config_file.device_name,
            phys: config_file.phys,
            mappings,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mapping {
    Remap {
        // these keys must be hold
        cond: HashSet<KeyCode>,
        // these keys must not be hold
        except: HashSet<KeyCode>,
        // this rule is triggered when one of these keys are pressed
        when: HashSet<KeyCode>,
        // .0 is remapped into .1 when active.
        // Deactivated when either a conflicting remapping is activated or .0 is released.
        mappings: Box<[(KeyCode, Box<[KeyCode]>)]>,
    },
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(try_from = "String")]
struct KeyCodeWrapper {
    pub code: KeyCode,
}

impl Into<KeyCode> for KeyCodeWrapper {
    fn into(self) -> KeyCode {
        self.code
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid key `{0}`.  Use `evremap list-keys` to see possible keys.")]
    InvalidKey(String),
    #[error("Impossible: parsed KEY_XXX but not into an EV_KEY")]
    ImpossibleParseKey,
}

impl std::convert::TryFrom<String> for KeyCodeWrapper {
    type Error = ConfigError;
    fn try_from(s: String) -> Result<KeyCodeWrapper, Self::Error> {
        match EventCode::from_str(&EventType::EV_KEY, &s) {
            Some(code) => match code {
                EventCode::EV_KEY(code) => Ok(KeyCodeWrapper { code }),
                _ => Err(ConfigError::ImpossibleParseKey),
            },
            None => Err(ConfigError::InvalidKey(s)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RemapConfig {
    cond: Vec<KeyCodeWrapper>,
    except: Vec<KeyCodeWrapper>,
    when: Vec<KeyCodeWrapper>,
    mappings: HashMap<KeyCodeWrapper, Vec<KeyCodeWrapper>>,
}

impl Into<Mapping> for RemapConfig {
    fn into(self) -> Mapping {
        Mapping::Remap {
            cond: self.cond.into_iter().map(Into::into).collect(),
            except: self.except.into_iter().map(Into::into).collect(),
            when: self.when.into_iter().map(Into::into).collect(),
            mappings: self
                .mappings
                .into_iter()
                .map(|(k, v)| (k.into(), v.into_iter().map(Into::into).collect()))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    device_name: String,
    #[serde(default)]
    phys: Option<String>,

    #[serde(default)]
    remap: Vec<RemapConfig>,
}
