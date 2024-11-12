use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub reset: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Error> {
        let cfg = std::fs::read_to_string(path).map_err(|e| Error::Config(e.to_string()))?;
        Self::from_yaml(&cfg).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub start_by_default: bool,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(default)]
    pub containers: Vec<Container>,
    pub network: Option<String>,
    pub image: Option<String>,
    pub command: Option<Vec<String>>,
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub environment: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Port {
    pub host: u16,
    pub container: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    #[serde(default)]
    pub command: Vec<String>,
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub environment: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<Volume>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Volume {
    pub host: String,
    pub container: String,
}

pub struct Test {
    pub path: String,

}

impl Config {
    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }
}
