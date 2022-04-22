## Cargo Structure

![Crates.io](https://img.shields.io/crates/v/cargo-structure?style=for-the-badge)

Cargo structure searches for all `Cargo.toml` files in your project, outputting a dot graph of the dependencies which can be fed into a renderer such as graphviz.

### Installation

```
cargo install cargo-structure
```

### Usage

```
cargo structure
```

Specific dependencies can also be ignored with the `--ignore` option.

```
cargo structure --ignore clap toml
```

### Contributions

Contributions are always welcome! Simply fork the repo and submit a pull request.
