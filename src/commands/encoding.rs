use rhai::{Dynamic, EvalAltResult, Position};

pub fn parse_json(json: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    serde_json::from_str(json).map_err(|e| {
        let msg = format!("Failed to parse JSON: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn parse_yaml(yaml: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    serde_yaml::from_str(yaml).map_err(|e| {
        let msg = format!("Failed to parse YAML: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn parse_toml(toml: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    toml::from_str(toml).map_err(|e| {
        let msg = format!("Failed to parse TOML: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn to_json(value: &Dynamic) -> Result<String, Box<EvalAltResult>> {
    serde_json::to_string(value).map_err(|e| {
        let msg = format!("Failed to convert to JSON: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn to_json_pretty(value: &Dynamic) -> Result<String, Box<EvalAltResult>> {
    serde_json::to_string_pretty(value).map_err(|e| {
        let msg = format!("Failed to convert to JSON: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn to_yaml(value: &Dynamic) -> Result<String, Box<EvalAltResult>> {
    serde_yaml::to_string(value).map_err(|e| {
        let msg = format!("Failed to convert to YAML: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn to_toml(value: &Dynamic) -> Result<String, Box<EvalAltResult>> {
    toml::to_string(value).map_err(|e| {
        let msg = format!("Failed to convert to TOML: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}
