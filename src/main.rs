#[macro_use]
extern crate toml;

use std::fmt::Debug;
use std::io::Error;
use std::{env, fs, path::Path};

use toml::Value;
use walkdir::WalkDir;

macro_rules! debug {
    ($e:expr) => {
        println!("{:?}", $e)
    };
}

#[derive(Debug)]
struct PackageInfo {
    name: String,
    dependencies: Vec<String>,
}

fn main() -> Result<(), Error> {
    let default_path = String::from(".");

    let args: Vec<String> = env::args().collect();
    let path_root = args.get(1).unwrap_or(&default_path);

    if !Path::new(path_root).exists() {
        println!("Root path not found");
        std::process::exit(-1);
    }

    let paths: Vec<String> = WalkDir::new(path_root)
        .into_iter()
        .map(|w| w.unwrap().path().display().to_string())
        .collect();

    let toml_file_paths: Vec<String> = paths
        .iter()
        .filter(|path| path.ends_with(".toml"))
        .cloned()
        .collect();

    let string_tomls: Vec<String> = toml_file_paths
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect();

    let parsed_tomls: Vec<Value> = string_tomls
        .iter()
        .filter_map(|t| toml::from_str::<Value>(t.as_str()).ok())
        .collect();

    let package_infos = parsed_tomls.iter().filter_map(|cargo| {
        let package_name = cargo
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(|str| str.as_str());
        let dependencies = cargo
            .get("dependencies")
            .and_then(|value| value.as_table())
            .and_then(|table| Some(table.keys().cloned().collect()));
        match (package_name, dependencies) {
            (Some(_package_name), Some(_dependencies)) => Some(PackageInfo {
                name: _package_name.to_string(),
                dependencies: _dependencies,
            }),
            _ => None,
        }
    });

    debug!(toml_file_paths);
    debug!(package_infos.collect::<Vec<_>>());

    Ok(())
}
