use std::{net::IpAddr, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
  #[arg(short, long, default_value = "0.0.0.0")]
  pub interface: IpAddr,

  #[arg(short, long, default_value_t = 3000)]
  pub port: u16,

  #[arg(short, long, default_value = "sensors")]
  pub sensor_dir: PathBuf,

  #[arg(short, long, default_value = "commands")]
  pub command_dir: PathBuf,
}
