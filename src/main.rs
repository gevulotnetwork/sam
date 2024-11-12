mod commands;
mod config;
mod environment;
mod rhai;

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
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Podman(e) => write!(f, "Podman error: {}", e),
            Self::Other(e) => write!(f, "Other error: {}", e),
            Self::Config(e) => write!(f, "Config error: {}", e),
            Self::Process(e) => write!(f, "Process error: {}", e),
        }
    }
}

fn setup_command_line_args() -> Command {
    clap::command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("run")
                .about("Run the e2e test environment")
                .arg(
                    clap::Arg::new("script")
                        .short('s')
                        .long("script")
                        .default_value("tests")
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
                ),
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
    if sub_matches.get_flag("reset-once") {
        reset_environment(sub_matches).await?;
    }

    let cfg = Config::load(sub_matches.get_one::<String>("config").unwrap())?;

    let mut env = ConfigurableEnvironment::new(cfg);
    env.start().await?;

    if let Some(delay) = sub_matches.get_one::<String>("delay") {
        log::info!("Delaying start of the tests by {}", delay);
        let duration = humantime::parse_duration(delay)
            .map_err(|e| Error::Other(format!("Failed to parse duration: {}", e)))?;
        tokio::time::sleep(duration).await;
    }

    let repeat = sub_matches
        .get_one::<u64>("repeat")
        .map(|r| *r)
        .unwrap_or(1);
    if repeat > 1 {
        log::info!("Repeating the tests {} times", repeat);
    }

    let module_dir = sub_matches
        .get_one::<String>("module-dir")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            let path = PathBuf::from(sub_matches.get_one::<String>("script").unwrap());
            path.parent().unwrap().to_string_lossy().into_owned()
        });

    let mut engine = Engine::new(env, &module_dir);

    if let Some(filter) = sub_matches.get_one::<String>("filter") {
        engine.set_filter(filter.to_string());
    }

    if let Some(skip) = sub_matches.get_one::<String>("skip") {
        engine.set_skip(skip.to_string());
    }

    for _ in 0..repeat {
        if let Some(script) = sub_matches.get_one::<String>("script") {
            match engine
                .run(PathBuf::from(script))
                .map_err(|e| Error::Other(e.to_string()))
            {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Script failed: {}", e);
                    return Err(e);
                }
            };
        }
    }

    if sub_matches.get_flag("keep-running") {
        log::info!("Press Ctrl-C to stop");
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
    }

    Ok(())
}

async fn reset_environment(sub_matches: &ArgMatches) -> Result<(), Error> {
    log::info!("Resetting environment");

    let cfg = Config::load(sub_matches.get_one::<String>("config").unwrap())?;
    for command in cfg.reset.iter() {
        tokio::process::Command::new("sh")
            .args(["-c", &command])
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

    match cmd.get_matches().subcommand() {
        Some(("run", sub_matches)) => run_environment(sub_matches).await?,
        Some(("reset", sub_matches)) => reset_environment(sub_matches).await?,
        _ => unreachable!("Subcommand required"),
    }

    Ok(())
}
