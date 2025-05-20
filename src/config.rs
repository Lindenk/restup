use std::{net::IpAddr, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
  #[arg(short, long, default_value = "0.0.0.0")]
  pub interface: IpAddr,

  #[arg(short, long, default_value_t = 3000)]
  pub port: u16,

  #[arg(short, long)]
  pub sensor_dir: Option<PathBuf>,

  #[arg(short, long)]
  pub command_dir: Option<PathBuf>,
}
