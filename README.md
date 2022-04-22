## Cargo Structure

[![Crates.io](https://img.shields.io/crates/v/cargo-structure?style=for-the-badge)](https://crates.io/crates/cargo-structure)
[![GitHub](https://img.shields.io/github/license/ramon54321/cargo-structure?style=for-the-badge)](https://github.com/ramon54321/cargo-structure/blob/main/LICENSE)

Cargo structure searches for all `Cargo.toml` files in your project, outputting a dot graph of the dependencies which can be fed into a renderer such as graphviz.

### Installation

```
cargo install cargo-structure
```

### Usage

Run cargo structure as a cargo plugin. The root package does not need to be speficied and will default to `.`, the current directory.

```
cargo structure <ROOT PACKAGE PATH>
```

Specific dependencies can also be ignored with the `--ignore` option.

```
cargo structure --ignore clap toml
```

If you have subcrates in your crate, they can be filtered out with a fuzzy search over their relative file path.

```
cargo structure --ignore-paths my_local_subcrate
```

### Contributions

Contributions are always welcome! Simply fork the repo and submit a pull request.
