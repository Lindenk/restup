use core::str;
use std::path::PathBuf;

use axum::{
  Router,
  extract::{Path, State},
  routing::{get, post},
};
use clap::Parser;
use log::info;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::process::Command;
use tower_http::services::ServeDir;

use error::Result;

mod config;
mod error;

#[tokio::main]
async fn main() {
  // init logging
  if let Err(e) = TermLogger::init(
    log::LevelFilter::Info,
    Config::default(),
    TerminalMode::Mixed,
    ColorChoice::Auto,
  ) {
    println!("Failed to initialize logging, {}", e);
  }

  // init cli args
  let args = config::Args::parse();

  // set up our routes
  let app = Router::new().route("/", get(|| async { "OK" }));

  let app = if let Some(dir) = args.sensor_dir.clone() {
    app.nest_service("/sensors/raw", ServeDir::new(dir))
  } else {
    app
  };

  let app = if args.command_dir.is_some() {
    app.route("/commands/raw/{*command}", post(run_command))
  } else {
    app
  };

  let app = app.with_state(args.clone());

  // bind our network interface
  let listener = tokio::net::TcpListener::bind((args.interface, args.port))
    .await
    .expect("Failed to bind to interface");

  // start the server
  if let Ok(addr) = listener.local_addr() {
    info!("Serving on {:?}", addr);
  };
  axum::serve(listener, app)
    .await
    .expect("Failed to serve endpoints");
}

/// Runs the executable at the path given relative to the base `command_dir`
async fn run_command(
  State(config): State<config::Args>,
  Path(command): Path<PathBuf>,
) -> Result<String> {
  // TODO: Parse our args into something that can't fail
  // Safety: `command_dir` is checked for None before binding this route
  let cmd_path = config.command_dir.unwrap().join(command);

  info!("Running command {:?}", cmd_path);
  let res = Command::new(cmd_path).output().await?.stdout;
  let res = str::from_utf8(res.as_slice()).unwrap();

  Ok(res.into())
}
