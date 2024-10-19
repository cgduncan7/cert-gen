use std::fs::File;
use std::io;
use std::io::prelude::*;
use serde_derive::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub title: String,
    pub fonts: Vec<Font>,
    pub texts: Vec<Text>,
}

#[derive(Deserialize)]
pub struct Font {
    pub name: String,
    pub file: String,
}

#[derive(Deserialize)]
pub struct Text {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub font: String,
    pub size: f32,
    pub conditional: Option<String>,
}

pub fn read_config(working_dir: &str) -> io::Result<Config> {
    let filename = format!("{}/config.toml", working_dir);
    let f = File::open(filename)?;
    let mut reader = io::BufReader::new(f);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let config: Config = toml::from_str(&buf).unwrap();

    Ok(config)
}