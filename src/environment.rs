use std::{collections::HashSet, process::Stdio};

use tokio::process::Command;

use crate::{config::Config, Error};

pub trait Environment: Send + Sync {
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    async fn start_component(&mut self, component_name: &str) -> Result<(), Error>;
    async fn stop_component(&mut self, component_name: &str) -> Result<(), Error>;
    fn stop_on_drop(&mut self, stop_on_drop: bool);
}

pub struct MockEnvironment {}
impl Environment for MockEnvironment {
    async fn start(&mut self) -> Result<(), Error> { Ok(()) }
    async fn stop(&mut self) -> Result<(), Error> { Ok(()) }
    async fn start_component(&mut self, _component_name: &str) -> Result<(), Error> { Ok(()) }
    async fn stop_component(&mut self, _component_name: &str) -> Result<(), Error> { Ok(()) }
    fn stop_on_drop(&mut self, _stop_on_drop: bool) {}
}

#[derive(Clone)]
pub struct ConfigurableEnvironment {
    cfg: Config,
    is_running: HashSet<String>,
    stop_on_drop: bool,
}

impl ConfigurableEnvironment {
    pub fn new(cfg: &Config) -> Self {
        Self {
            cfg: cfg.clone(),
            is_running: HashSet::new(),
            stop_on_drop: true,
        }
    }

    async fn make_sure_network_exists(&self) -> Result<(), Error> {
        let output = Command::new("podman")
            .arg("network")
            .arg("exists")
            .arg("samnet")
            .output()
            .await
            .map_err(|e| Error::Podman(e.to_string()))?;
        if !output.status.success() {
            log::info!("Creating podman network samnet");
            Command::new("podman")
                .arg("network")
                .arg("create")
                .arg("samnet")
                .output()
                .await
                .map_err(|e| Error::Podman(e.to_string()))?;
        }
        Ok(())
    }

    async fn start_component_with_deps(&mut self, component_name: &str) -> Result<(), Error> {
        // Get all dependencies recursively
        let mut deps = std::collections::HashSet::new();
        let mut queue = vec![component_name.to_string()];

        while let Some(comp) = queue.pop() {
            if let Some(component) = self.cfg.get_component(&comp) {
                for dep in &component.dependencies {
                    if !self.is_running.contains(dep) && deps.insert(dep.clone()) {
                        queue.push(dep.clone());
                    }
                }
            }
        }

        // Start dependencies in order
        let mut started = std::collections::HashSet::new();
        let mut remaining: Vec<_> = deps.into_iter().collect();

        while !remaining.is_empty() {
            let mut made_progress = false;

            remaining.retain(|dep_name| {
                let component = self
                    .cfg
                    .get_component(dep_name)
                    .unwrap_or_else(|| panic!("Component {} not found in config", dep_name));

                // Check if all dependencies are started
                let deps_satisfied = component
                    .dependencies
                    .iter()
                    .all(|dep| started.contains(dep));

                if deps_satisfied {
                    // Start this component
                    if let Err(e) = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(self.start_component(dep_name))
                    }) {
                        log::error!("Failed to start component {}: {}", dep_name, e);
                        return true; // Keep in remaining list
                    }

                    log::debug!("Started dependency {}", dep_name);
                    started.insert(dep_name.clone());
                    made_progress = true;
                    false // Remove from remaining list
                } else {
                    log::debug!("Component {} waiting for dependencies", dep_name);
                    true // Keep in remaining list
                }
            });

            if !made_progress && !remaining.is_empty() {
                return Err(Error::Config(format!(
                    "Circular dependency detected in components: {:?}",
                    remaining
                )));
            }
        }

        // Finally start the requested component
        ConfigurableEnvironment::start_component(self, component_name).await?;

        Ok(())
    }

    async fn start_component(&mut self, component_name: &str) -> Result<(), Error> {
        if self.is_running.contains(component_name) {
            log::debug!("Component {} already running, skipping", component_name);
            return Ok(());
        }

        log::debug!("Starting component {}", component_name);

        let component = self.cfg.get_component(component_name).ok_or_else(|| {
            Error::Config(format!("Component {} not found in config", component_name))
        })?;

        match component.component_type.as_str() {
            "container" => {
                // Start container here
                let mut cmd = Command::new("podman");
                cmd.arg("run")
                    .arg("-d")
                    .arg("--replace")
                    .arg("--name")
                    .arg(&component.name);

                // Add volumes if specified
                for volume in &component.volumes {
                    cmd.arg("-v")
                        .arg(format!("{}:{}:z", volume.host, volume.container));
                }

                // Add environment variables if specified
                for env in &component.environment {
                    cmd.arg("-e").arg(env);
                }

                // Add network mode if specified
                if let Some(network) = &component.network {
                    cmd.arg(format!("--network={}", network));
                }

                // Add ports if specified
                for port in &component.ports {
                    cmd.arg("-p")
                        .arg(format!("{}:{}", port.host, port.container));
                }

                // Add entrypoint if specified
                if let Some(entrypoint) = &component.entrypoint {
                    cmd.arg("--entrypoint").arg(entrypoint);
                }

                // Add image
                cmd.arg(component.image.as_ref().ok_or_else(|| {
                    Error::Config(format!("Image not specified for component {:?}", component))
                })?);

                // Add command if specified
                if let Some(command) = &component.command {
                    cmd.args(command);
                }

                let output = cmd
                    .output()
                    .await
                    .map_err(|e| Error::Podman(e.to_string()))?;

                if !output.status.success() {
                    return Err(Error::Podman(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }
            }
            "pod" => {
                self.make_sure_network_exists().await?;

                let pod_name = &component.name;

                // Create pod
                let mut cmd = Command::new("podman");
                cmd.arg("pod")
                    .arg("create")
                    .arg("--replace")
                    .arg("--name")
                    .arg(pod_name);

                let network_arg = format!(
                    "--network={}",
                    &component.network.as_ref().unwrap_or(&"samnet".to_string())
                );
                cmd.arg(network_arg);

                // Add port mappings if specified
                for port in &component.ports {
                    cmd.arg(format!("-p={}:{}", port.host, port.container));
                }

                let output = cmd
                    .output()
                    .await
                    .map_err(|e| Error::Podman(e.to_string()))?;

                if !output.status.success() {
                    return Err(Error::Podman(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }

                // Start all containers in the pod
                for container in &component.containers {
                    let mut cmd = Command::new("podman");
                    cmd.arg("run")
                        .arg("-d")
                        .arg("--pod")
                        .arg(pod_name)
                        .arg("--name")
                        .arg(&container.name);

                    // Add volumes if specified
                    for volume in &container.volumes {
                        cmd.arg("-v")
                            .arg(format!("{}:{}", volume.host, volume.container));
                    }

                    // Add environment variables if specified
                    for env in &container.environment {
                        cmd.arg("-e").arg(env);
                    }

                    // Add entrypoint if specified
                    if let Some(entrypoint) = &container.entrypoint {
                        cmd.arg("--entrypoint").arg(entrypoint);
                    }

                    if let Some(network) = &container.network {
                        cmd.arg(format!("--network={}", network));
                    }

                    cmd.arg(&container.image);

                    // Add command if specified
                    if !container.command.is_empty() {
                        for arg in &container.command {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd
                        .output()
                        .await
                        .map_err(|e| Error::Podman(e.to_string()))?;

                    if !output.status.success() {
                        return Err(Error::Podman(
                            String::from_utf8_lossy(&output.stderr).to_string(),
                        ));
                    }
                }
            }
            "process" => {
                let command = component.command.as_ref().ok_or_else(|| {
                    Error::Config(format!(
                        "Command not specified for component {:?}",
                        component
                    ))
                })?;
                if command.is_empty() {
                    return Err(Error::Config(format!(
                        "Command is empty for component {:?}",
                        component
                    )));
                }

                let mut cmd = Command::new(&command[0]);

                if command.len() > 1 {
                    // Add arguments
                    cmd.args(&command[1..]);
                }

                cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

                let child = cmd.spawn().map_err(|e| Error::Process(e.to_string()))?;

                // Write PID to file
                if let Some(pid) = child.id() {
                    let runtime_dir =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
                    let pid_file_path =
                        std::path::Path::new(&runtime_dir).join(format!("{}.pid", component_name));
                    std::fs::write(&pid_file_path, pid.to_string())
                        .map_err(|e| Error::Process(e.to_string()))?;
                }

                // Handle stdout
                if let Some(mut stdout) = child.stdout {
                    let runtime_dir =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
                    let stdout_file = std::path::Path::new(&runtime_dir)
                        .join(format!("{}.stdout", component_name));
                    tokio::spawn(async move {
                        let mut file = tokio::fs::File::create(&stdout_file).await.unwrap();
                        tokio::io::copy(&mut stdout, &mut file).await.unwrap();
                    });
                }

                // Handle stderr
                if let Some(mut stderr) = child.stderr {
                    let runtime_dir =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
                    let stderr_file = std::path::Path::new(&runtime_dir)
                        .join(format!("{}.stderr", component_name));
                    tokio::spawn(async move {
                        let mut file = tokio::fs::File::create(&stderr_file).await.unwrap();
                        tokio::io::copy(&mut stderr, &mut file).await.unwrap();
                    });
                }
            }
            _ => {
                return Err(Error::Config(format!(
                    "Unknown component type: {}",
                    component.component_type
                )))
            }
        }

        self.is_running.insert(component_name.to_string());

        Ok(())
    }

    async fn stop_component(&mut self, component_name: &str) -> Result<(), Error> {
        log::debug!("Stopping component {}", component_name);

        if !self.is_running.contains(component_name) {
            log::debug!("Component {} not running, skipping", component_name);
            return Ok(());
        }

        let component = self.cfg.get_component(component_name).ok_or_else(|| {
            Error::Config(format!("Component {} not found in config", component_name))
        })?;

        match component.component_type.as_str() {
            "pod" => {
                let pod_name = &component.name;

                let output = Command::new("podman")
                    .arg("pod")
                    .arg("rm")
                    .arg("-f")
                    .arg("-t=0")
                    .arg(pod_name)
                    .output()
                    .await
                    .map_err(|e| Error::Podman(e.to_string()))?;

                if !output.status.success() {
                    return Err(Error::Podman(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }
            }
            "container" => {
                let container_name = &component.name;

                let output = Command::new("podman")
                    .arg("rm")
                    .arg("-f")
                    .arg("-t=0")
                    .arg(container_name)
                    .output()
                    .await
                    .map_err(|e| Error::Podman(e.to_string()))?;

                if !output.status.success() {
                    return Err(Error::Podman(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }
            }
            "process" => {
                // Read PID from file
                let runtime_dir =
                    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
                let pid_file_path =
                    std::path::Path::new(&runtime_dir).join(format!("{}.pid", component_name));
                let pid = std::fs::read_to_string(&pid_file_path)
                    .map_err(|e| Error::Process(e.to_string()))?;

                // Kill process
                if let Err(e) = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(&pid)
                    .output()
                {
                    log::error!("Failed to kill process {}: {}", pid, e);
                    return Err(Error::Process(e.to_string()));
                }
            }
            _ => {
                return Err(Error::Config(format!(
                    "Unknown component type: {}",
                    component.component_type
                )))
            }
        }

        self.is_running.remove(component_name);

        Ok(())
    }
}

impl Environment for ConfigurableEnvironment {
    async fn start(&mut self) -> Result<(), Error> {
        log::info!("Starting environment...");
        let start_time = std::time::Instant::now(); // Start timing

        // Start all components in dependency order
        let mut started = std::collections::HashSet::new();

        let mut remaining: Vec<_> = self
            .cfg
            .components
            .iter()
            .filter(|c| c.start_by_default)
            .map(|c| c.name.clone())
            .collect();

        while !remaining.is_empty() {
            let mut made_progress = false;

            remaining.retain(|component_name| {
                let component = self.cfg.get_component(component_name).unwrap();

                // Check if all dependencies are started
                let deps_satisfied = component
                    .dependencies
                    .iter()
                    .all(|dep| started.contains(dep));

                if deps_satisfied {
                    // Start this component
                    if let Err(e) = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(self.start_component(component_name))
                    }) {
                        log::error!("Failed to start component {}: {}", component_name, e);
                        return true; // Keep in remaining list
                    }

                    log::debug!("Started component {}", component_name);
                    started.insert(component_name.clone());
                    made_progress = true;
                    false // Remove from remaining list
                } else {
                    log::debug!("Component {} waiting for dependencies", component_name);
                    true // Keep in remaining list
                }
            });

            if !made_progress && !remaining.is_empty() {
                return Err(Error::Config(format!(
                    "Something went wrong while starting components: {:?}",
                    remaining
                )));
            }
        }

        let duration = start_time.elapsed(); // Calculate elapsed time
        log::info!(
            "Environment started successfully in {}",
            humantime::format_duration(duration)
        );
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        log::info!("Stopping environment...");

        let stop_time = std::time::Instant::now(); // Start timing

        // Stop all components in reverse dependency order
        let mut stopped: std::collections::HashSet<String> = self
            .cfg
            .components
            .iter()
            .map(|c| c.name.clone())
            .filter(|name| !self.is_running.contains(name))
            .collect();
        let mut remaining: Vec<_> = self.is_running.iter().cloned().collect();

        while !remaining.is_empty() {
            let mut made_progress = false;

            remaining.retain(|component_name| {
                // Check if all dependents are stopped
                let can_stop = self
                    .cfg
                    .components
                    .iter()
                    .filter(|c| c.dependencies.contains(component_name))
                    .all(|c| stopped.contains(&c.name));

                if can_stop {
                    // Stop this component
                    if let Err(e) = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(self.stop_component(component_name))
                    }) {
                        log::error!("Failed to stop component {}: {}", component_name, e);
                        return true; // Keep in remaining list
                    }

                    log::debug!("Stopped component {}", component_name);
                    stopped.insert(component_name.clone());
                    made_progress = true;
                    false // Remove from remaining list
                } else {
                    log::debug!(
                        "Component {} waiting for dependents to stop",
                        component_name
                    );
                    true // Keep in remaining list
                }
            });

            if !made_progress && !remaining.is_empty() {
                return Err(Error::Config(format!(
                    "Circular dependency detected while stopping components: {:?}",
                    remaining
                )));
            }
        }

        // Remove all pods
        log::debug!("Removing pods");
        for pod in self
            .cfg
            .components
            .iter()
            .filter(|c| c.component_type == "pod")
        {
            log::debug!("Removing pod {}", pod.name);
            let output = Command::new("podman")
                .arg("pod")
                .arg("rm")
                .arg("-f")
                .arg("-t=0")
                .arg(&pod.name)
                .output()
                .await
                .map_err(|e| Error::Podman(e.to_string()))?;

            if !output.status.success() {
                return Err(Error::Podman(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
            log::debug!("Removed pod {}", pod.name);
        }

        let duration = stop_time.elapsed(); // Calculate elapsed time
        log::info!(
            "Environment stopped successfully in {}",
            humantime::format_duration(duration)
        );
        Ok(())
    }

    async fn start_component(&mut self, component_name: &str) -> Result<(), Error> {
        self.start_component_with_deps(component_name).await
    }

    async fn stop_component(&mut self, component_name: &str) -> Result<(), Error> {
        ConfigurableEnvironment::stop_component(self, component_name).await
    }

    fn stop_on_drop(&mut self, stop_on_drop: bool) {
        self.stop_on_drop = stop_on_drop;
    }
}

impl Drop for ConfigurableEnvironment {
    fn drop(&mut self) {
        if self.stop_on_drop {
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(self.stop())
            });
        }
    }
}
