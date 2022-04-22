use clap::{Arg, ArgMatches, Command};
use std::fmt::Debug;
use std::io::Error;
use std::{fs, path::Path};
use toml::Value;
use walkdir::WalkDir;

#[allow(unused_macros)]
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
    let arguments = get_arguments();
    let path_root = get_path_root(&arguments);
    let paths = get_paths(&arguments, &path_root);
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

    let package_infos = get_package_infos(&arguments, &parsed_tomls);
    let dot_string = get_dot_string_from_package_infos(&package_infos);

    println!("{}", dot_string);

    Ok(())
}

fn get_arguments() -> ArgMatches {
    Command::new("Cargo Structure")
        .version("0.1.0")
        .author("Ramon Brand")
        .about("A utility for analyzing the structure of a cargo project.")
        .no_binary_name(false)
        .arg(Arg::new("structure").hide(true))
        .arg(
            Arg::new("root")
                .value_name("ROOT PACKAGE")
                .long_help("The path to the root package. This path contains the parent Cargo.toml."),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("PACKAGES")
                .multiple_values(true)
                .takes_value(true)
                .long_help("Multiple optional package names which will be ignored.")
        )
        .arg(
            Arg::new("ignore-paths")
                .short('I')
                .long("ignore-paths")
                .value_name("FUZZY QUERY")
                .multiple_values(true)
                .takes_value(true)
                .long_help("Multiple optional strings which will be used to filter out paths of child packages.")
        )
        .get_matches()
}

fn get_path_root(arguments: &ArgMatches) -> String {
    let default_path = String::from(".");
    let path_root = arguments
        .value_of("root")
        .unwrap_or(&default_path)
        .to_string();
    if !Path::new(&path_root).exists() {
        println!("Root path does not exist: {}", path_root);
        std::process::exit(-1);
    }
    path_root
}

fn get_paths(arguments: &ArgMatches, path_root: &String) -> Vec<String> {
    let paths: Vec<String> = WalkDir::new(path_root)
        .into_iter()
        .map(|w| w.unwrap().path().display().to_string())
        .collect();

    let paths = if !arguments.is_present("ignore-paths") {
        paths
    } else {
        let fuzzy_ignores: Vec<&str> = arguments.values_of("ignore-paths").unwrap().collect();
        paths
            .into_iter()
            .filter(|path| {
                !fuzzy_ignores
                    .iter()
                    .any(|fuzzy_ignore| path.contains(fuzzy_ignore))
            })
            .collect()
    };

    paths
}

fn get_package_infos(arguments: &ArgMatches, parsed_tomls: &Vec<Value>) -> Vec<PackageInfo> {
    let package_infos: Vec<PackageInfo> = parsed_tomls
        .iter()
        .filter_map(|cargo| {
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
        })
        .collect();

    let package_infos = if !arguments.is_present("ignore") {
        package_infos
    } else {
        let ignores: Vec<String> = arguments
            .values_of("ignore")
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        package_infos
            .into_iter()
            .filter(|package_info| !ignores.contains(&package_info.name))
            .map(|package_info| PackageInfo {
                dependencies: package_info
                    .dependencies
                    .into_iter()
                    .filter(|dependency| !ignores.contains(dependency))
                    .collect(),
                ..package_info
            })
            .collect()
    };

    package_infos
}

fn get_dot_string_from_package_infos(package_infos: &Vec<PackageInfo>) -> String {
    let mut dot_bytes = Vec::new();
    {
        let mut writer = dot_writer::DotWriter::from(&mut dot_bytes);
        writer.set_pretty_print(false);
        let mut graph = writer.digraph();
        package_infos.iter().for_each(|package_info| {
            package_info.dependencies.iter().for_each(|dependency| {
                let name = format!("\"{}\"", package_info.name.clone());
                let dependency = format!("\"{}\"", dependency);
                graph.edge(name, dependency);
            })
        });
    }
    let dot_string = String::from_utf8(dot_bytes);
    match dot_string {
        Ok(_dot_string) => _dot_string,
        _ => std::process::exit(-2),
    }
}
