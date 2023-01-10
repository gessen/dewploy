mod cli;

use crate::cli::{Args, BuildType, DaemonType};
use anyhow::{bail, Result};
use clap::Parser;
use std::{net::Ipv4Addr, path::PathBuf, process::Command};

const TARGET_DIR: &str = "target-deploy";

fn main() -> Result<()> {
    let args = Args::parse();

    let daemon_type = parse_daemon_type(&args)?;
    let build_type = parse_build_type(&args)?;
    let ip = parse_ip(&args)?;
    let cross_build = parse_cross_build(&args);
    let only_daemon = args.only_daemon;
    let only_runner = args.only_runner;
    let no_strip = args.no_strip;
    let no_stop = args.no_stop;
    let no_start = args.no_start;
    let keep_logs = args.keep_logs;
    let working_dir = parse_working_dir(&args);

    switch_to_working_dir(working_dir)?;

    if !no_stop {
        stop_stormcloud(ip)?;
    }

    deploy_project(
        build_type,
        daemon_type,
        ip,
        cross_build,
        only_daemon,
        only_runner,
        no_strip,
    )?;

    if !keep_logs {
        remove_logs(ip)?;
    }

    if !no_start {
        start_stormcloud(ip)?;
    }

    Ok(())
}

fn parse_daemon_type(args: &Args) -> Result<DaemonType> {
    if let Some(daemon_type) = args.daemon_type {
        return Ok(daemon_type);
    }

    if let Ok(daemon_type_env) = std::env::var("STORMCLOUD_DAEMON_TYPE") {
        if let Ok(daemon_type) = daemon_type_env.parse() {
            return Ok(daemon_type);
        }
    }

    bail!("STORMCLOUD_DAEMON_TYPE env var must be defined or --daemon-type must be supplied");
}

fn parse_build_type(args: &Args) -> Result<BuildType> {
    if let Some(build_type) = args.build_type {
        return Ok(build_type);
    }

    if let Ok(build_type_env) = std::env::var("STORMCLOUD_BUILD_TYPE") {
        if let Ok(build_type) = build_type_env.parse() {
            return Ok(build_type);
        }
    }

    bail!("STORMCLOUD_BUILD_TYPE env var must be defined or --build-type must be supplied");
}

fn parse_ip(args: &Args) -> Result<Ipv4Addr> {
    if let Some(ip) = args.ip {
        return Ok(ip);
    }

    if let Ok(ip_env) = std::env::var("GHOST_IP") {
        if let Ok(ip) = ip_env.parse() {
            return Ok(ip);
        }
    }

    bail!("GHOST_IP env var must be defined or --ip must be supplied");
}

fn parse_cross_build(args: &Args) -> bool {
    if std::env::var("STORMCLOUD_CROSS_BUILD").is_ok() {
        return true;
    }

    args.cross_build
}

fn parse_working_dir(args: &Args) -> Option<PathBuf> {
    if let Some(working_dir) = &args.working_dir {
        return Some(working_dir.clone());
    }

    if let Ok(working_dir_env) = std::env::var("STORMCLOUD_DIR") {
        return Some(PathBuf::from(working_dir_env));
    }

    None
}

fn switch_to_working_dir(working_dir: Option<PathBuf>) -> Result<()> {
    if let Some(working_dir) = working_dir {
        std::env::set_current_dir(working_dir)?
    }

    Ok(())
}

fn deploy_project(
    build_type: BuildType,
    daemon_type: DaemonType,
    ip: Ipv4Addr,
    cross_build: bool,
    only_daemon: bool,
    only_runner: bool,
    no_strip: bool,
) -> Result<()> {
    if !only_runner {
        build_daemon(build_type, daemon_type, cross_build)?;
    }

    if !only_daemon {
        build_runner(build_type, cross_build)?;
    }

    if !no_strip {
        if !only_runner {
            strip_daemon(build_type, daemon_type)?;
        }
        if !only_daemon {
            strip_runner(build_type)?;
        }
    }

    if !only_runner {
        upload_daemon(build_type, daemon_type, ip)?;
    }

    if !only_daemon {
        upload_runner(build_type, ip)?;
    }

    Ok(())
}

fn stop_stormcloud(ip: Ipv4Addr) -> Result<()> {
    let mut command = create_stop_command(ip);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!("failed to stop stormcloud on {ip}");
    }

    Ok(())
}

fn start_stormcloud(ip: Ipv4Addr) -> Result<()> {
    let mut command = create_start_command(ip);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!("failed to start stormcloud on {ip}");
    }

    Ok(())
}

fn remove_logs(ip: Ipv4Addr) -> Result<()> {
    let mut command = create_remove_logs_command(ip);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!("failed to remove logs on {ip}");
    }
    Ok(())
}

fn build_daemon(build_type: BuildType, daemon_type: DaemonType, cross_build: bool) -> Result<()> {
    let mut command = create_build_command(cross_build);

    command.arg("--package");
    command.arg("stormcloud_daemon");

    if let DaemonType::Sync = daemon_type {
        command.arg("--features");
        command.arg(daemon_type.to_string().to_lowercase());
    }

    let target_dir = match daemon_type {
        DaemonType::Async => TARGET_DIR.to_string(),
        DaemonType::Sync => format!("{}-sync", TARGET_DIR),
    };

    command.arg("--target-dir");
    command.arg(target_dir);

    if let BuildType::Release = build_type {
        command.arg("--release");
    }

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to build {} {} daemon",
            build_type.to_string().to_lowercase(),
            daemon_type.to_string().to_lowercase()
        );
    }

    Ok(())
}

fn build_runner(build_type: BuildType, cross_build: bool) -> Result<()> {
    let mut command = create_build_command(cross_build);

    command.arg("--package");
    command.arg("stormrunner_javascript");

    command.arg("--target-dir");
    command.arg(TARGET_DIR);

    if let BuildType::Release = build_type {
        command.arg("--release");
    }

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to build {} runner",
            build_type.to_string().to_lowercase()
        );
    }

    Ok(())
}

fn strip_daemon(build_type: BuildType, daemon_type: DaemonType) -> Result<()> {
    let target_file = format!(
        "{}/{}/stormcloud_daemon",
        match daemon_type {
            DaemonType::Async => TARGET_DIR.to_string(),
            DaemonType::Sync => format!("{}-sync", TARGET_DIR),
        },
        build_type.to_string().to_lowercase()
    );

    let mut command = create_strip_command();
    command.arg(target_file);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to strip {} {} daemon",
            build_type.to_string().to_lowercase(),
            daemon_type.to_string().to_lowercase(),
        )
    }

    Ok(())
}

fn strip_runner(build_type: BuildType) -> Result<()> {
    let target_file = format!(
        "{}/{}/stormrunner_javascript",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    );

    let mut command = create_strip_command();
    command.arg(target_file);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to strip {} runner",
            build_type.to_string().to_lowercase(),
        )
    }

    Ok(())
}

fn upload_daemon(build_type: BuildType, daemon_type: DaemonType, ip: Ipv4Addr) -> Result<()> {
    let source_file = format!(
        "{}/{}/stormcloud_daemon",
        match daemon_type {
            DaemonType::Async => TARGET_DIR.to_string(),
            DaemonType::Sync => format!("{}-sync", TARGET_DIR),
        },
        build_type.to_string().to_lowercase()
    );

    let target_file = format!(
        "root@{}:/a/stormcloud/{}/{}/stormcloud_daemon",
        ip,
        match daemon_type {
            DaemonType::Async => "bin",
            DaemonType::Sync => "binsync",
        },
        build_type.to_string().to_lowercase(),
    );

    let mut command = create_upload_command();
    command.arg(source_file);
    command.arg(target_file);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to upload {} {} daemon to {}",
            build_type.to_string().to_lowercase(),
            daemon_type.to_string().to_lowercase(),
            ip
        );
    }

    Ok(())
}

fn upload_runner(build_type: BuildType, ip: Ipv4Addr) -> Result<()> {
    let source_file = format!(
        "{}/{}/stormrunner_javascript",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    );

    let target_file = format!(
        "root@{}:/a/stormcloud/stormlets/{}/deployed/stormlet_javascript@0.0.0/stormrunner_javascript.0.0.0",
        ip,
        build_type.to_string().to_lowercase(),
    );

    let mut command = create_upload_command();
    command.arg(source_file);
    command.arg(target_file);

    pretty_print(&command);
    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to upload {} runner to {}",
            build_type.to_string().to_lowercase(),
            ip
        );
    }

    Ok(())
}

fn create_stop_command(ip: Ipv4Addr) -> Command {
    let mut command = Command::new("ssh");
    command
        .arg(format!("root@{}", ip))
        .arg("/a/sbin/akamai_run")
        .arg("stop")
        .arg("stormcloud");
    command
}

fn create_start_command(ip: Ipv4Addr) -> Command {
    let mut command = Command::new("ssh");
    command
        .arg(format!("root@{}", ip))
        .arg("/a/sbin/akamai_run")
        .arg("start")
        .arg("stormcloud");
    command
}

fn create_remove_logs_command(ip: Ipv4Addr) -> Command {
    let mut command = Command::new("ssh");
    command
        .arg(format!("root@{}", ip))
        .arg("rm")
        .arg("-rf")
        .arg("/a/logs/stormcloud");
    command
}

fn create_build_command(cross_build: bool) -> Command {
    let mut command = if cross_build {
        Command::new("cross")
    } else {
        Command::new("cargo")
    };
    command.arg("build");

    command
}

fn create_strip_command() -> Command {
    let mut command = Command::new("strip");
    command.arg("--strip-unneeded");

    command
}

fn create_upload_command() -> Command {
    let mut command = Command::new("rsync");
    command
        .arg("--human-readable")
        .arg("--compress")
        .arg("--progress")
        .arg("--verbose");

    command
}

fn pretty_print(command: &Command) {
    println!(
        "\x1b[1;33m{}\x1b[0m",
        format!("{:?}", command).replace('\"', "")
    );
}
