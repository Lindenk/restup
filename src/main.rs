use core::str;
use std::path::PathBuf;

use axum::{
  Router,
  extract::{Path, State},
  routing::{get, post},
};
use clap::Parser;
use log::info;
use tokio::process::Command;
use tower_http::services::ServeDir;

use error::Result;

mod config;
mod error;

#[tokio::main]
async fn main() {
  let args = config::Args::parse();

  let app = Router::new()
    .route("/", get(|| async { "OK" }))
    .nest_service("/sensors/raw", ServeDir::new(args.sensor_dir.clone()))
    .route("/commands/raw/{*command}", post(run_command))
    .with_state(args.clone());

  let listener = tokio::net::TcpListener::bind((args.interface, args.port))
    .await
    .expect("Failed to bind to interface");
  axum::serve(listener, app)
    .await
    .expect("Failed to serve endpoints");
}

async fn run_command(
  State(config): State<config::Args>,
  Path(command): Path<PathBuf>,
) -> Result<String> {
  let cmd_path = config.command_dir.join(command);
  info!("Running command {:?}", cmd_path);
  let res = Command::new(cmd_path).output().await?.stdout;
  let res = str::from_utf8(res.as_slice()).unwrap();

  Ok(res.into())
}
