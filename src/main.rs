mod commands;
mod config;
mod environment;
mod rhai;
mod state;

use std::path::PathBuf;

use clap::{ArgMatches, Command};
use config::Config;
use environment::*;
use rhai::Engine;

#[derive(Debug)]
enum Error {
    Podman(String),
    Other(String),
    Config(String),
    Process(String),
    Test(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Podman(e) => write!(f, "Podman error: {}", e),
            Self::Other(e) => write!(f, "Other error: {}", e),
            Self::Config(e) => write!(f, "Config error: {}", e),
            Self::Process(e) => write!(f, "Process error: {}", e),
            Self::Test(e) => write!(f, "Test error: {}", e),
        }
    }
}

fn setup_command_line_args() -> Command {
    clap::command!()
        .arg(
            clap::Arg::new("script")
                .short('s')
                .long("script")
                .action(clap::ArgAction::Append)
                .help("Test script or directory"),
        )
        .arg(
            clap::Arg::new("keep-running")
                .short('k')
                .long("keep-running")
                .action(clap::ArgAction::SetTrue)
                .help("Keep the environment running after the script has finished"),
        )
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .default_value("config.yaml")
                .help("Use a custom config file"),
        )
        .arg(
            clap::Arg::new("delay")
                .short('d')
                .long("delay")
                .help("Delay the start of the tests"),
        )
        .arg(
            clap::Arg::new("repeat")
                .short('r')
                .long("repeat")
                .value_parser(clap::value_parser!(u64))
                .help("Repeat the script"),
        )
        .arg(
            clap::Arg::new("filter")
                .short('f')
                .long("filter")
                .help("Filter the tests"),
        )
        .arg(
            clap::Arg::new("skip")
                .short('x')
                .long("skip")
                .help("Skip the tests"),
        )
        .arg(
            clap::Arg::new("reset-once")
                .long("reset-once")
                .action(clap::ArgAction::SetTrue)
                .help("Reset the environment once before starting up"),
        )
        .arg(
            clap::Arg::new("force")
                .long("force")
                .action(clap::ArgAction::SetTrue)
                .help("Force reset the environment"),
        )
        .arg(
            clap::Arg::new("module-dir")
                .long("module-dir")
                .help("The directory containing the Rhai modules"),
        )
        .arg(
            clap::Arg::new("output")
                .long("output")
                .short('o')
                .help("The file to output the test report to"),
        )
        .subcommand(
            Command::new("reset")
                .about("Reset the e2e test environment")
                .arg(
                    clap::Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(clap::ArgAction::SetTrue)
                        .help("Force reset the environment"),
                )
                .arg(
                    clap::Arg::new("config")
                        .short('c')
                        .long("config")
                        .default_value("config.yaml")
                        .help("Use a custom config file"),
                ),
        )
}

async fn run_environment(sub_matches: &ArgMatches) -> Result<(), Error> {
    log::debug!("Starting run_environment");

    log::debug!("Loading config file");
    let mut cfg = Config::load(sub_matches.get_one::<String>("config").unwrap())?;
    cfg.read_flags(sub_matches)?;

    if cfg.global.reset_once {
        log::debug!("Reset-once flag detected, resetting environment");
        reset_environment(sub_matches).await?;
    }

    let global_cfg = cfg.global.clone();
    log::debug!("Creating configurable environment");
    let mut env = ConfigurableEnvironment::new(&cfg);

    log::debug!("Starting environment");
    env.start().await?;

    if let Some(delay) = global_cfg.delay {
        log::info!("Delaying start of the tests by {}", delay);
        log::debug!("Parsing delay duration: {}", delay);
        let duration = humantime::parse_duration(&delay)
            .map_err(|e| Error::Other(format!("Failed to parse duration: {}", e)))?;
        tokio::time::sleep(duration).await;
    }

    let repeat = global_cfg.repeat.unwrap_or(1);
    if repeat > 1 {
        log::info!("Repeating the tests {} times", repeat);
    }

    log::debug!("Setting up module directories");
    let mut module_dirs = global_cfg.module_dirs.clone();
    if module_dirs.is_empty() {
        log::debug!("No module directories specified, using script directory");
        let first_script = global_cfg
            .scripts
            .first()
            .ok_or(Error::Config("No scripts found in config".to_string()))?;
        let path = PathBuf::from(first_script);
        if path.is_file() {
            log::debug!("Using parent directory of script file: {}", first_script);
            module_dirs.push(
                path.parent()
                    .ok_or(Error::Other(format!(
                        "No parent directory found for script {}",
                        first_script
                    )))?
                    .to_string_lossy()
                    .into_owned(),
            );
        } else if path.is_dir() {
            log::debug!("Using script directory directly: {}", first_script);
            module_dirs.push(path.to_string_lossy().into_owned());
        } else {
            return Err(Error::Other(format!(
                "No script or directory found at {}",
                first_script
            )));
        }
    }

    log::debug!(
        "Creating Rhai engine with module directories: {:?}",
        module_dirs
    );
    let mut engine = Engine::new(env, &module_dirs);

    if let Some(filter) = &global_cfg.filter {
        log::debug!("Setting filter: {}", filter);
        engine.set_filter(filter.to_string());
    }

    if let Some(skip) = &global_cfg.skip {
        log::debug!("Setting skip: {}", skip);
        engine.set_skip(skip.to_string());
    }

    for i in 0..repeat {
        log::debug!("Starting iteration {} of {}", i + 1, repeat);
        for script in &global_cfg.scripts {
            match engine
                .run(PathBuf::from(script))
                .map_err(|e| Error::Other(e.to_string()))
            {
                Ok(_) => log::debug!("Script {} completed successfully", script),
                Err(e) => {
                    log::error!("Script {} failed: {}", script, e);
                    return Err(e);
                }
            };
        }
    }

    if sub_matches.get_flag("keep-running") {
        log::info!("Press Ctrl-C to stop");
        log::debug!("Waiting for Ctrl-C signal");
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
    }


    if let Some(output) = sub_matches.get_one::<String>("output") {
        log::debug!("Writing test report to {}", output);
        let report = engine.get_report();
        let is_yaml = output.ends_with(".yaml") || output.ends_with(".yml");
        if is_yaml {
            std::fs::write(output, serde_yaml::to_string(&report).unwrap())
                .map_err(|e| Error::Other(e.to_string()))?;
        } else {
            std::fs::write(output, serde_json::to_string_pretty(&report).unwrap())
                .map_err(|e| Error::Other(e.to_string()))?;
        }
    }
    if engine.get_error_count() > 0 {
        return Err(Error::Test(format!(
            "Test run failed with {} failed assertions",
            engine.get_error_count()
        )));
    }

    log::debug!("run_environment completed successfully");
    Ok(())
}

async fn reset_environment(sub_matches: &ArgMatches) -> Result<(), Error> {
    log::info!("Resetting environment");

    let cfg = Config::load(sub_matches.get_one::<String>("config").unwrap())?;
    for command in cfg.reset.iter() {
        tokio::process::Command::new("sh")
            .args(["-c", command])
            .spawn()
            .map_err(|e| Error::Other(e.to_string()))?
            .wait()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
    }

    Ok(())
}

struct TimeLogger {
    start: std::time::Instant,
}

impl Drop for TimeLogger {
    fn drop(&mut self) {
        log::info!(
            "Run completed in {}",
            humantime::format_duration(self.start.elapsed())
        );
    }
}

fn welcome() {
    log::info!("                   _____ _____ _____ ");
    log::info!("                  |   __|  _  |     |");
    log::info!("                  |__   |     | | | |");
    log::info!("                  |_____|__|__|_|_|_|");
    log::info!("                Simple Automation Manager");
    log::info!("Welcome to SAM, your simple and friendly test automation manager!");
    log::info!("");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // will log the duration of the program
    let _time_logger = TimeLogger {
        start: std::time::Instant::now(),
    };

    pretty_env_logger::init();

    welcome();

    let cmd = setup_command_line_args();
    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("reset", sub_matches)) => reset_environment(sub_matches).await?,
        None => run_environment(&matches).await?,
        _ => unreachable!("Invalid subcommand"),
    }

    Ok(())
}
