use core::str;
use std::{collections::HashMap, path::PathBuf};

use axum::{
  Json, Router,
  extract::{Path, State},
  routing::{get, post},
};
use clap::Parser;
use log::{info, warn};
use serde::Serialize;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::{fs::File, io::AsyncReadExt, process::Command};
use tower_http::services::ServeDir;

use error::Result;
use walkdir::WalkDir;

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
  let app = Router::new().route("/", get(|| async { "on" }));

  let app = if let Some(dir) = args.sensor_dir.clone() {
    app
      .nest_service("/sensors/raw", ServeDir::new(dir))
      .route("/sensors", get(get_sensors_json))
  } else {
    warn!("No sensor directory specified, skipping. (you can set it with the -s flag)");
    app
  };

  let app = if args.command_dir.is_some() {
    app.route("/commands/raw/{*command}", post(run_command))
  } else {
    warn!("No command directory specified, skipping. (you can set it with the -c flag)");
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

/// Representation of the the files in a directory
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum SensorData {
  Text(String),
  Directory(HashMap<String, SensorData>),
}

/// Returns the entire sensor directory's files in json format
async fn get_sensors_json(State(config): State<config::Args>) -> Json<HashMap<String, SensorData>> {
  // Safety: This is checked for existance before adding this route
  // TODO: Don't let the sensor_dir be None when passed in
  let sensor_dir = config.sensor_dir.unwrap();

  // Make our json respresentation to be filled
  let mut sensors: HashMap<String, SensorData> = HashMap::new();

  // for every regular file, insert it's contents into the nested hashmap using it's path as keys
  for sensor in WalkDir::new(sensor_dir.clone()).into_iter() {
    // validate we're only reading valid and accessable files
    let Ok(sensor) = sensor else {
      log::error!("Failed accessing sensor: {:?}", sensor);
      continue;
    };

    if sensor.file_type().is_file() {
      let mut f = match File::open(sensor.path()).await {
        Ok(f) => f,
        Err(e) => {
          warn!("Unable to open sensor {:?}: {}", sensor, e);
          continue;
        }
      };

      let mut value = String::default();
      if let Err(e) = f.read_to_string(&mut value).await {
        warn!("Unable to read sensor {:?}: {}", f, e);
        continue;
      };

      // remove the base prefix and split the path into it's directories and file name
      let path = sensor
        .path()
        .strip_prefix(&sensor_dir)
        .expect("sensor_dir prefix somehow was not the prefix for the file paths it contains");

      let mut path: Vec<String> = path
        .to_string_lossy()
        .split('/')
        .map(String::from)
        .collect();

      let file = path
        .pop()
        .expect("Walkdir somehow returned a file without a name");

      // recurse down the nested hashmaps until we find the correct leaf node for this file, and insert it
      let mut current_table = &mut sensors;
      for d in path.iter() {
        let entry = current_table.entry(d.clone());
        let entry = entry.or_insert(SensorData::Directory(HashMap::default()));

        match entry {
          SensorData::Directory(m) => current_table = m,
          _ => {
            panic!("Somehow tried using file {:?} as a directory", path);
          }
        }
      }

      current_table.insert(file.to_string(), SensorData::Text(value));
    }
  }

  Json(sensors)
}
