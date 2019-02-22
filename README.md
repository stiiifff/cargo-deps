# cargo-deps

[![](https://img.shields.io/crates/v/cargo-deps.svg)](https://crates.io/crates/cargo-deps) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![LoC](https://tokei.rs/b1/github/m-cat/cargo-deps)](https://github.com/m-cat/cargo-deps)

Cargo subcommand for building dependency graphs of Rust projects.

This project is an improvement on the unmaintained and buggy [cargo-graph](https://github.com/kbknapp/cargo-graph).

## Installing

`cargo-deps` can be installed with `cargo install`:

```
cargo install cargo-deps
```

## Instructions

First, make sure you have [graphviz](https://graphviz.gitlab.io/download/) installed.

Next, just `cd` into the Rust project directory you want to graph and run:

```
cargo deps | dot -Tpng >| graph.png
```

That's it! `graph.png` will contain the graph (you can change its name, of course!)

### Settings

The default behavior is to exclude optional, dev, and build dependencies. To see all dependencies, pass `--all-deps`:

```
cargo deps --all-deps | dot -Tpng >| graph.png
```

Dependencies are colored depending on their kind:

* **Black:** regular dependency
* **Purple:** build dependency
* **Blue:** dev dependency
* **Red:** optional dependency

A dependency can be of more than one kind. In such cases, it is colored with the following priority:

```
Regular -> Build -> Dev -> Optional
```

For example, if a dependency is both a build and a dev dependency, then it will be colored as a build dependency. If, however, you pass the `--dev-deps` option instead of `--all-deps`, the dependency will be colored as a dev dependency (as the build-dependency graph will not be shown).

### Example

**Tokei:** [no regular dependencies](tokei.png)

This was generated using the command:

```
cargo deps -I --all-deps --no-regular-deps | dot -Tpng >| tokei.png
```

### More info

Run `cargo deps -h` to see all available options.

## License

`cargo-deps` is released under the terms of the MIT license. See the [LICENSE-MIT](./LICENSE-MIT) file for the details.

## Dependencies

![cargo-deps dependencies](cargo-deps.png)
