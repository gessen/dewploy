use clap::{ArgEnum, ArgGroup, Parser};
use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("only").args(&["only-daemon", "only-runner"])))]
pub struct Args {
    /// Use cross to build Stormcloud project
    #[clap(long, short)]
    pub cross_build: bool,

    /// Do not strip built binaries
    #[clap(long)]
    pub no_strip: bool,

    /// Type of the optimisations
    #[clap(long, short, arg_enum, value_name = "TYPE")]
    pub build_type: Option<BuildType>,

    /// Type of the daemon messaging
    #[clap(long, short = 't', arg_enum, value_name = "TYPE")]
    pub daemon_type: Option<DaemonType>,

    /// IP of the ghost
    #[clap(long, short, parse(try_from_str), value_name = "IPv4")]
    pub ip: Option<Ipv4Addr>,

    /// Build and upload only Stormcloud Daemon
    #[clap(long, short = 'd')]
    pub only_daemon: bool,

    /// Build and upload only Stormrunner Javascript
    #[clap(long, short = 'r')]
    pub only_runner: bool,
}

#[derive(Clone, Copy, Debug, ArgEnum)]
pub enum BuildType {
    Debug,
    Release,
}

#[derive(Clone, Copy, Debug, ArgEnum)]
pub enum DaemonType {
    Async,
    Sync,
}

impl FromStr for BuildType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(BuildType::Debug),
            "release" => Ok(BuildType::Release),
            _ => Err(()),
        }
    }
}

impl Display for BuildType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for DaemonType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "async" => Ok(DaemonType::Async),
            "sync" => Ok(DaemonType::Sync),
            _ => Err(()),
        }
    }
}

impl Display for DaemonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
