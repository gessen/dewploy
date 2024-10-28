mod cli;

use crate::cli::{Args, BuildType};
use anyhow::{bail, Result};
use clap::Parser;
use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    process::Command,
};

const TARGET_DIR: &str = "target-deploy";

fn main() -> Result<()> {
    let args = Args::parse();

    let build_type = parse_build_type(&args)?;
    let ip = parse_ip(&args)?;
    let Args {
        only_daemon,
        only_runner,
        with_cloudbuster,
        no_stop,
        no_start,
        keep_logs,
        no_strip,
        working_dir,
        ..
    } = args;

    switch_to_working_dir(working_dir)?;

    if !no_stop {
        stop_stormcloud(ip)?;
    }

    deploy_project(
        build_type,
        ip,
        only_daemon,
        only_runner,
        with_cloudbuster,
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

fn switch_to_working_dir(working_dir: Option<PathBuf>) -> Result<()> {
    if let Some(working_dir) = working_dir {
        std::env::set_current_dir(working_dir)?
    }

    Ok(())
}

fn deploy_project(
    build_type: BuildType,
    ip: Ipv4Addr,
    only_daemon: bool,
    only_runner: bool,
    with_cloudbuster: bool,
    no_strip: bool,
) -> Result<()> {
    if !only_runner {
        build_daemon(build_type)?;
    }

    if !only_daemon {
        build_runner(build_type)?;
    }

    if with_cloudbuster {
        build_cloudbuster(build_type)?;
    }

    if !no_strip {
        if !only_runner {
            strip_daemon(build_type)?;
        }
        if !only_daemon {
            strip_runner(build_type)?;
        }
        if with_cloudbuster {
            strip_cloudbuster(build_type)?;
        }
    }

    if !only_runner {
        upload_daemon(build_type, ip)?;
    }

    if !only_daemon {
        upload_runner(build_type, ip)?;
    }

    if with_cloudbuster {
        upload_cloudbuster(build_type, ip)?;
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

fn build_daemon(build_type: BuildType) -> Result<()> {
    let mut command = create_build_command("stormcloud_daemon", build_type);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to build {} daemon",
            build_type.to_string().to_lowercase(),
        );
    }

    Ok(())
}

fn build_runner(build_type: BuildType) -> Result<()> {
    let mut command = create_build_command("stormrunner_javascript", build_type);
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

fn build_cloudbuster(build_type: BuildType) -> Result<()> {
    let mut command = create_build_command("cloudbuster", build_type);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to build {} cloudbuster",
            build_type.to_string().to_lowercase()
        );
    }

    Ok(())
}

fn strip_daemon(build_type: BuildType) -> Result<()> {
    let target = PathBuf::from(format!(
        "{}/{}/stormcloud_daemon",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let debug = PathBuf::from(format!("{}.debug", target.to_string_lossy()));

    let mut command = create_copy_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to copy debuginfo for {} daemon",
            build_type.to_string().to_lowercase(),
        )
    }

    let mut command = create_remove_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to remove debuginfo for {} daemon",
            build_type.to_string().to_lowercase(),
        )
    }

    Ok(())
}

fn strip_runner(build_type: BuildType) -> Result<()> {
    let target = PathBuf::from(format!(
        "{}/{}/stormrunner_javascript",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let debug = PathBuf::from(format!("{}.debug", target.to_string_lossy()));

    let mut command = create_copy_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to copy debuginfo for {} runner",
            build_type.to_string().to_lowercase(),
        )
    }

    let mut command = create_remove_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to remove debuginfo for {} runner",
            build_type.to_string().to_lowercase(),
        )
    }

    Ok(())
}

fn strip_cloudbuster(build_type: BuildType) -> Result<()> {
    let target = PathBuf::from(format!(
        "{}/{}/cloudbuster",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let debug = PathBuf::from(format!("{}.debug", target.to_string_lossy()));

    let mut command = create_copy_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to copy debuginfo for {} cloudbuster",
            build_type.to_string().to_lowercase(),
        )
    }

    let mut command = create_remove_debuginfo_command(&target, &debug);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to remove debuginfo for {} cloudbuster",
            build_type.to_string().to_lowercase(),
        )
    }

    Ok(())
}

fn upload_daemon(build_type: BuildType, ip: Ipv4Addr) -> Result<()> {
    let source = PathBuf::from(format!(
        "{}/{}/stormcloud_daemon",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let target = PathBuf::from(format!(
        "root@{}:/a/stormcloud/bin/release/stormcloud_daemon",
        ip
    ));

    let mut command = create_upload_command(&source, &target);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to upload {} daemon to {}",
            build_type.to_string().to_lowercase(),
            ip
        );
    }

    Ok(())
}

fn upload_runner(build_type: BuildType, ip: Ipv4Addr) -> Result<()> {
    let source = PathBuf::from(format!(
        "{}/{}/stormrunner_javascript",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let target = PathBuf::from(format!(
        "root@{}:/a/stormcloud/stormlets/release/deployed/stormlet_javascript@0.0.0/stormrunner_javascript.0.0.0",
        ip,
    ));

    let mut command = create_upload_command(&source, &target);
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

fn upload_cloudbuster(build_type: BuildType, ip: Ipv4Addr) -> Result<()> {
    let source = PathBuf::from(format!(
        "{}/{}/cloudbuster",
        TARGET_DIR,
        build_type.to_string().to_lowercase()
    ));
    let target = PathBuf::from(format!("root@{}:/a/stormcloud/bin/cloudbuster", ip));

    let mut command = create_upload_command(&source, &target);
    pretty_print(&command);

    let status = command.status()?;
    if !status.success() {
        bail!(
            "failed to upload {} cloudbuster to {}",
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

fn create_build_command(package: &str, build_type: BuildType) -> Command {
    let mut command = Command::new("cross");
    command
        .arg("build")
        .arg("--target-dir")
        .arg(TARGET_DIR)
        .arg("--package")
        .arg(package);
    if let BuildType::Release = build_type {
        command.arg("--release");
    }
    command
}

fn create_copy_debuginfo_command(target: &Path, debug: &Path) -> Command {
    let mut command = Command::new("objcopy");
    command.arg("--only-keep-debug").arg(target).arg(debug);
    command
}

fn create_remove_debuginfo_command(target: &Path, debug: &Path) -> Command {
    let mut command = Command::new("objcopy");
    command
        .arg("--strip-debug")
        .arg("--strip-unneeded")
        .arg("--remove-section")
        .arg(".gnu_debuglink")
        .arg("--add-gnu-debuglink")
        .arg(debug.file_name().unwrap_or_default())
        .arg(target.file_name().unwrap_or_default());
    command.current_dir(target.parent().unwrap_or(Path::new(TARGET_DIR)));
    command
}

fn create_upload_command(source: &Path, target: &Path) -> Command {
    let mut command = Command::new("rsync");
    command
        .arg("--human-readable")
        .arg("--compress")
        .arg("--progress")
        .arg("--verbose")
        .arg(source)
        .arg(target);
    command
}

fn pretty_print(command: &Command) {
    println!(
        "\x1b[1;33m{}\x1b[0m",
        format!("{:?}", command).replace('\"', "")
    );
}
