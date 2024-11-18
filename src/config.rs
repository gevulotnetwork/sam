use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub base: Option<String>,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub reset: Vec<String>,
    #[serde(default)]
    pub global: Global,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Default)]
pub struct Global {
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub keep_running: bool,
    pub delay: Option<String>,
    pub repeat: Option<u64>,
    pub filter: Option<String>,
    pub skip: Option<String>,
    #[serde(default)]
    pub reset_once: bool,
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub module_dirs: Vec<String>
}


impl Config {
    pub fn load(path: &str) -> Result<Self, Error> {
        let cfg = std::fs::read_to_string(path).map_err(|e| Error::Config(e.to_string()))?;
        let mut cfg = Self::from_yaml(&cfg).map_err(|e| Error::Config(e.to_string()))?;
        if let Some(base) = &cfg.base {
            let base_cfg = Self::load(base)?;
            cfg = base_cfg.merge(&cfg)?;
        }
        Ok(cfg)
    }

    pub fn merge(&self, other: &Self) -> Result<Self, Error> {
        let mut result = self.clone();
        for component in &other.components {
            if let Some(pos) = result.components.iter().position(|c| c.name == component.name) {
                result.components[pos] = component.clone();
            } else {
                result.components.push(component.clone());
            }
        }

        // Merge global settings
        if !other.global.scripts.is_empty() {
            result.global.scripts = other.global.scripts.clone();
        }
        if !other.global.module_dirs.is_empty() {
            result.global.module_dirs = other.global.module_dirs.clone();
        }
        if other.global.delay.is_some() {
            result.global.delay = other.global.delay.clone();
        }
        if other.global.repeat.is_some() {
            result.global.repeat = other.global.repeat;
        }
        if other.global.filter.is_some() {
            result.global.filter = other.global.filter.clone();
        }
        if other.global.skip.is_some() {
            result.global.skip = other.global.skip.clone();
        }
        result.global.reset_once |= other.global.reset_once;
        result.global.force |= other.global.force;
        result.global.keep_running |= other.global.keep_running;

        Ok(result)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn read_flags(&mut self, args: &ArgMatches) -> Result<(), Error> {
        if let Some(scripts) = args.get_many::<String>("script") {
            let all_scripts: Vec<String> = scripts.map(|s| s.to_string()).collect();
            if !all_scripts.is_empty() {
                log::debug!("Setting scripts from command line: {:?}", all_scripts);
                self.global.scripts = all_scripts;
            }
        }

        if let Some(delay) = args.get_one::<String>("delay") {
            log::debug!("Setting delay from command line: {}", delay);
            self.global.delay = Some(delay.to_string());
        }

        if let Some(repeat) = args.get_one::<u64>("repeat") {
            log::debug!("Setting repeat from command line: {}", repeat);
            self.global.repeat = Some(*repeat);
        }

        if let Some(filter) = args.get_one::<String>("filter") {
            log::debug!("Setting filter from command line: {}", filter);
            self.global.filter = Some(filter.to_string());
        }

        if let Some(skip) = args.get_one::<String>("skip") {
            log::debug!("Setting skip from command line: {}", skip);
            self.global.skip = Some(skip.to_string());
        }

        if let Some(module_dirs) = args.get_many::<String>("module-dir") {
            let dirs: Vec<String> = module_dirs.map(|s| s.to_string()).collect();
            log::debug!("Setting module directories from command line: {:?}", dirs);
            self.global.module_dirs = dirs;
        }

        if args.get_flag("keep-running") {
            log::debug!("Setting keep_running from command line: true");
            self.global.keep_running = true;
        }

        if args.get_flag("reset-once") {
            log::debug!("Setting reset_once from command line: true");
            self.global.reset_once = true;
        }

        if args.get_flag("force") {
            log::debug!("Setting force from command line: true");
            self.global.force = true;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Port {
    pub host: u16,
    pub container: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
    pub host: String,
    pub container: String,
}

impl Config {
    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }
}
