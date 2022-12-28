use clap::{Arg, ArgMatches, Command};
use std::collections::HashSet;
use std::fmt::Debug;
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

fn main() -> Result<(), i32> {
    let arguments = get_arguments();
    let path_root = get_path_root(&arguments);

    let applicable_tomls: Vec<Value> = if arguments.is_present("monolithic") {
        get_parsed_tomls_monolithic(&arguments, &path_root)
    } else {
        get_parsed_tomls_recursive(&arguments, &path_root)
    }
    .ok_or(-1)?;

    if applicable_tomls.is_empty() {
        println!("No Cargo.toml found in {}", path_root);
        std::process::exit(-1);
    }

    let package_infos = get_package_infos(&arguments, &applicable_tomls);
    let dot_string = get_dot_string_from_package_infos(&package_infos);

    println!("{}", dot_string);

    Ok(())
}

fn get_parsed_toml_at_path(path_root: &String) -> Option<Value> {
    let paths: Vec<String> = fs::read_dir(path_root)
        .ok()?
        .filter_map(|directory| Some(directory.ok()?.path().display().to_string()))
        .collect();
    let parsed_tomls: Vec<Value> = paths
        .into_iter()
        .filter(|path| path.ends_with(".toml"))
        .filter_map(|p| fs::read_to_string(p).ok())
        .filter_map(|t| toml::from_str::<Value>(t.as_str()).ok())
        .collect();
    Some(parsed_tomls.first()?.clone())
}

fn get_parsed_tomls_recursive(arguments: &ArgMatches, path_root: &String) -> Option<Vec<Value>> {
    let parsed_toml = get_parsed_toml_at_path(&path_root)
        .expect(&format!("Unable to get parsed toml at path {}", &path_root));
    let members = get_parsed_toml_workspace_members(&parsed_toml);
    let dependencies = get_parsed_toml_dependencies(&parsed_toml);
    let members = if members.is_some() {
        members.unwrap()
    } else {
        Vec::new()
    };
    let dependencies = if dependencies.is_some() {
        dependencies.unwrap()
    } else {
        Vec::new()
    };
    let member_paths: Vec<String> = members
        .iter()
        .filter_map(|value| Some(value.as_str()?.to_string()))
        .collect();
    let local_dependency_relative_paths: Vec<String> = dependencies
        .iter()
        .filter_map(|(_, value)| Some(value.get("path")?.as_str()?.to_string()))
        .collect();
    let all_paths = {
        let mut paths = member_paths.clone();
        let mut paths_deps = local_dependency_relative_paths.clone();
        paths.append(&mut paths_deps);
        paths
    };
    let child_toml_paths: Vec<String> = all_paths
        .iter()
        .map(|relative_path| Path::new(path_root).join(relative_path))
        .map(|path| path.display().to_string())
        .collect();
    let parsed_toml = vec![parsed_toml];
    if child_toml_paths.is_empty() {
        Some(parsed_toml)
    } else {
        let child_parsed_tomls: Vec<Value> = child_toml_paths
            .iter()
            .filter_map(|child_path| get_parsed_tomls_recursive(arguments, child_path))
            .flatten()
            .collect();
        let all_parsed_tomls = parsed_toml
            .iter()
            .cloned()
            .chain(child_parsed_tomls.iter().cloned())
            .collect();
        Some(all_parsed_tomls)
    }
}

fn get_parsed_toml_workspace_members(parsed_toml: &Value) -> Option<Vec<Value>> {
    Some(
        parsed_toml
            .get("workspace")?
            .as_table()?
            .get("members")?
            .as_array()?
            .to_owned(),
    )
}

fn get_parsed_toml_dependencies(parsed_toml: &Value) -> Option<Vec<(String, Value)>> {
    Some(
        parsed_toml
            .get("dependencies")?
            .as_table()?
            .iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect::<Vec<(String, Value)>>()
            .to_owned(),
    )
}

fn get_parsed_tomls_monolithic(arguments: &ArgMatches, path_root: &String) -> Option<Vec<Value>> {
    let paths = get_paths_to_all_non_ignored_sub_files(&arguments, &path_root);
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
    Some(parsed_tomls)
}

fn get_arguments() -> ArgMatches {
    Command::new("Cargo Structure")
        .version("0.4.1")
        .author("Ramon Brand")
        .about("A utility for analyzing the structure of a cargo project.")
        .no_binary_name(false)
        .arg(Arg::new("structure").hide(true))
        .arg(
            Arg::new("root")
                .value_name("ROOT PACKAGE PATH")
                .long_help("The path to the root package. This path contains the parent Cargo.toml."),
        )
        .arg(
            Arg::new("monolithic")
                .short('m')
                .long("monolithic")
                .long_help("Treat all child Cargo.toml files as part of the same dependency graph.")
        )
        .arg(
            Arg::new("local")
                .short('l')
                .long("local")
                .long_help("Include only local subcrates.")
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
                .requires("monolithic")
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

fn get_paths_to_all_non_ignored_sub_files(
    arguments: &ArgMatches,
    path_root: &String,
) -> Vec<String> {
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
    let package_infos = if arguments.is_present("local") {
        let local_package_names: Vec<String> = parsed_tomls
            .iter()
            .filter_map(|toml| Some(toml.get("package")?.get("name")?.as_str()?.to_string()))
            .collect();
        package_infos
            .into_iter()
            .filter(|package_info| local_package_names.contains(&package_info.name))
            .map(|package_info| PackageInfo {
                dependencies: package_info
                    .dependencies
                    .into_iter()
                    .filter(|dependency| local_package_names.contains(dependency))
                    .collect(),
                ..package_info
            })
            .collect()
    } else {
        package_infos
    };
    let package_infos = if arguments.is_present("ignore") {
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
    } else {
        package_infos
    };
    package_infos
}

fn get_dot_string_from_package_infos(package_infos: &Vec<PackageInfo>) -> String {
    let mut dot_bytes = Vec::new();
    {
        let mut writer = dot_writer::DotWriter::from(&mut dot_bytes);
        writer.set_pretty_print(false);
        let mut edges = HashSet::new();
        let mut graph = writer.digraph();
        package_infos.iter().for_each(|package_info| {
            package_info.dependencies.iter().for_each(|dependency| {
                let name = format!("\"{}\"", package_info.name.clone());
                let dependency = format!("\"{}\"", dependency);
                let edge_tuple = (name.clone(), dependency.clone());
                if !edges.contains(&edge_tuple) {
                    edges.insert(edge_tuple);
                    graph.edge(name, dependency);
                }
            })
        });
    }
    let dot_string = String::from_utf8(dot_bytes);
    match dot_string {
        Ok(_dot_string) => _dot_string,
        _ => std::process::exit(-2),
    }
}
