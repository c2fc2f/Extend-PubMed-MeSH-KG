# xpmkg — Extend PubMed-MeSH Knowledge Graph

A command-line multitool written in Rust for enriching PubMed-MeSH knowledge graphs (CSV-based, targeting Neo4j) with additional nodes, relationships, and external metadata.

It is designed to extend the output produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG), adding nodes, relationships, or properties sourced from external biomedical databases such as UMLS.

## Overview

The project is organized as a Cargo workspace with one internal library crate:

- **`crates/dispatch_derive`** — procedural macro that derives an async `dispatch` method on clap subcommand enums

The `xpmkg` binary ties the subcommands together.

## Requirements

- Rust toolchain (edition 2024, stable)
- Output CSVs produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG)
- A locally extracted UMLS release (for subcommands that consume UMLS data)

## Installation

### From source

```bash
git clone https://github.com/c2fc2f/Extend-PubMed-MeSH-KG
cd Extend-PubMed-MeSH-KG
cargo build --release
# or
cargo install --git https://github.com/c2fc2f/Extend-PubMed-MeSH-KG
```

The compiled binary will be at `target/release/xpmkg`.

### With Nix

A Nix flake is provided:

```bash
nix run github:c2fc2f/Extend-PubMed-MeSH-KG -- --help
# or
nix build
# or, to enter a development shell:
nix develop
```

## Usage

```
xpmkg <COMMAND>
```

Run `xpmkg --help` for the full list of available subcommands, or `xpmkg <COMMAND> --help` for subcommand-specific options.

## Subcommands

| Subcommand | Description | Documentation |
|---|---|---|
| `cui-umls` | Annotates MeSH nodes with their corresponding UMLS CUIs | [README](src/subcommand/cui_umls/README.md) |
| `kg-umls` | Creates cross-graph relationships between MeSH elements and UMLS atoms and concepts | [README](src/subcommand/kg_umls/README.md) |

## Library Crate

The `dispatch_derive` library crate can be used independently in other projects.

### `dispatch_derive`

An internal procedural macro crate that derives an async `dispatch` method on clap subcommand enums. It eliminates the boilerplate of hand-written `match` arms by generating dispatch from the enum variant's type path to the corresponding `run` function.

See [`crates/dispatch_derive/README.md`](crates/dispatch_derive/README.md) for full documentation.

## License

This project is licensed under the [MIT License](LICENSE).
