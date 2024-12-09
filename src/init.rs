use std::{fs, path::Path};

use clap::ArgMatches;

use crate::Error;

pub async fn init(sub_matches: &ArgMatches) -> Result<(), Error> {
    // Create directory structure
    let dirs = ["tests/cases", "tests/modules", "tests/assets"];
    for dir in dirs {
        if !Path::new(dir).exists() || sub_matches.get_flag("force") {
            log::info!("Creating directory {}", dir);
            fs::create_dir_all(dir).map_err(|e| Error::Config(e.to_string()))?;
            if dir == "tests/assets" {
                log::info!("Creating example asset file tests/assets/hello.txt");
                fs::write("tests/assets/hello.txt", "hello world").map_err(|e| Error::Config(e.to_string()))?;
            }
        } else {
            log::info!("Directory {} already exists, skipping", dir);
        }
    }

    // Create example config file
    let config_path = "sam.yaml";
    if !Path::new(config_path).exists() || sub_matches.get_flag("force") {
        log::info!("Creating example config file {}", config_path);
        let example_config = r#"name: example-test
global:
  scripts:
    - tests/cases/example.rhai
  module_dirs:
    - tests/modules
  delay: null
  repeat: null
  filter: null
  skip: null
  reset_once: false
  force: false
  keep_running: false
components:
  - name: caddy
    type: container
    start_by_default: true
    image: docker.io/library/caddy:latest
    command: ["caddy", "file-server", "--root", "/srv"]
    ports:
      - host: 8080
        container: 80
    volumes:
      - host: ./tests/assets
        container: /srv
    environment:
      - CADDY_ADMIN_PORT=2019
reset:
  - echo 'Reverts assets...'
  - echo 'hello world' > tests/assets/hello.txt"#;
        fs::write(config_path, example_config).map_err(|e| Error::Config(e.to_string()))?;
    } else {
        log::info!("Config file {} already exists, skipping", config_path);
    }

    // Create example module
    let module_path = "tests/modules/example.rhai";
    if !Path::new(module_path).exists() || sub_matches.get_flag("force") {
        log::info!("Creating example module {}", module_path);
        let example_module = r#"// Example test module

// explicitly export constants
export const URL = "http://127.0.0.1:8080/hello.txt";

// functions are always exported
fn fetch(url) {
    http_get(#{url: url});
}

"#;
        fs::write(module_path, example_module).map_err(|e| Error::Config(e.to_string()))?;
    } else {
        log::info!("Module file {} already exists, skipping", module_path);
    }

    // Create example test case
    let test_path = "tests/cases/example.rhai";
    if !Path::new(test_path).exists() || sub_matches.get_flag("force") {
        log::info!("Creating example test case {}", test_path);
        let example_test = r#"import "example" as example;

describe("Example Test Suite", || {
    it("should respond to HTTP requests", || {
        // Make request to Caddy server
        let response = example::fetch(example::URL);
        
        // Validate response
        assert(response == "hello world", "Expected valid response from server");
    });
});"#;
        fs::write(test_path, example_test).map_err(|e| Error::Config(e.to_string()))?;
    } else {
        log::info!("Test file {} already exists, skipping", test_path);
    }

    log::info!("Initialization complete!");
    Ok(())
}
