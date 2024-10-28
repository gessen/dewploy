use clap::{Parser, ValueEnum, ValueHint};
use std::{fmt, net::Ipv4Addr, path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Type of the optimisations
    #[arg(long, short, value_enum, value_name = "TYPE")]
    pub build_type: Option<BuildType>,

    /// IP of the ghost
    #[arg(long, short, value_name = "IPv4")]
    pub ip: Option<Ipv4Addr>,

    /// Build and upload only Stormcloud Daemon
    #[arg(long, short = 'd', conflicts_with = "only_runner")]
    pub only_daemon: bool,

    /// Build and upload only Stormrunner Javascript
    #[arg(long, short = 'r', conflicts_with = "only_daemon")]
    pub only_runner: bool,

    /// Additionaly build and upload Cloudbuster
    #[arg(long)]
    pub with_cloudbuster: bool,

    /// Do not stop Stormcloud before deploying
    #[arg(long)]
    pub no_stop: bool,

    /// Do not start Stormcloud after deploying
    #[arg(long)]
    pub no_start: bool,

    /// Do not remove older Stormcloud logs
    #[arg(long)]
    pub keep_logs: bool,

    /// Do not strip built binaries
    #[arg(long)]
    pub no_strip: bool,

    /// Swap to this dir before building Stormcloud
    #[arg(long, short = 'C', value_hint = ValueHint::DirPath, value_name = "DIR")]
    pub working_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BuildType {
    Debug,
    Release,
    FastRelease,
}

impl FromStr for BuildType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(BuildType::Debug),
            "release" => Ok(BuildType::Release),
            "fast-release" => Ok(BuildType::FastRelease),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BuildType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildType::FastRelease => write!(f, "fast_release"),
            _ => write!(f, "{self:?}"),
        }
    }
}
