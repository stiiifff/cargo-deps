# cargo-deps

[![](https://img.shields.io/crates/v/cargo-deps.svg)](https://crates.io/crates/cargo-deps) [![Documentation](https://docs.rs/cargo-deps/badge.svg)](https://docs.rs/cargo-deps) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![LoC](https://tokei.rs/b1/github/m-cat/cargo-deps)](https://github.com/m-cat/cargo-deps)

Cargo subcommand for building dependency graphs of Rust projects.

This project is a fork of the unmaintained [cargo-graph](https://github.com/kbknapp/cargo-graph).

## Demo

Let's say we wanted to build a dependency graph of [cargo-count](https://github.com/kbknapp/cargo-count) but we wanted optional dependencies to use red dashed lines and black boxes, and regular (aka "build") dependencies to use orange lines to green diamonds, one would run the following.

**NOTE:** GraphViz `dot` needs to be installed to produce the .PNG from the dotfile

```
cargo deps --optional-line-style dashed --optional-line-color red --optional-shape box --build-shape diamond --build-color green --build-line-color orange > cargo-count.dot
dot -Tpng > rainbow-graph.png cargo-count.dot
```

**NOTE:** It's also possible to run `cargo deps [options] | dot [options] > [file]` instead of individual commands

The first command produces a GraphViz DOT file which looks like this:

```
digraph dependencies {
  N0[label="cargo-count",shape=diamond,color=green];
  N1[label="ansi_term",shape=box];
  N2[label="clap",shape=diamond,color=green];
  N3[label="clippy",shape=box];
  N4[label="glob",shape=diamond,color=green];
  N5[label="regex",shape=diamond,color=green];
  N6[label="tabwriter",shape=diamond,color=green];
  N7[label="aho-corasick",shape=diamond,color=green];
  N8[label="memchr",shape=diamond,color=green];
  N9[label="bitflags",shape=diamond,color=green];
  N10[label="strsim",shape=diamond,color=green];
  N11[label="unicode-normalization",shape=diamond,color=green];
  N12[label="libc",shape=diamond,color=green];
  N13[label="regex-syntax",shape=diamond,color=green];
  N14[label="unicode-width",shape=diamond,color=green];
  N0 -> N1[label="",style=dashed,color=red];
  N0 -> N2[label="",color=orange];
  N0 -> N3[label="",style=dashed,color=red];
  N0 -> N4[label="",color=orange];
  N0 -> N5[label="",color=orange];
  N0 -> N6[label="",color=orange];
  N7 -> N8[label="",color=orange];
  N2 -> N1[label="",style=dashed,color=red];
  N2 -> N9[label="",color=orange];
  N2 -> N10[label="",color=orange];
  N3 -> N11[label="",color=orange];
  N8 -> N12[label="",color=orange];
  N5 -> N7[label="",color=orange];
  N5 -> N8[label="",color=orange];
  N5 -> N13[label="",color=orange];
  N6 -> N14[label="",color=orange];
}
```

The second command produces a PNG image of the graph which looks like:

![cargo-count dependencies](rainbow-graph.png)

Now, *why* someone would do that to a graph is a different story... but it's possible :)

## Installing

`cargo-deps` can be installed with `cargo install`

```
cargo install cargo-deps
```

## Options

Runs `cargo deps -h` to see all the available options.

## License

`cargo-deps` is released under the terms of the MIT. See the [LICENSE-MIT](./LICENSE-MIT) file for the details.

## Dependencies

![cargo-deps dependencies](cargo-deps.png)
