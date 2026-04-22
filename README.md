# xpmkg — Extend PubMed-MeSH Knowledge Graph

A command-line multitool for enriching PubMed-MeSH knowledge graphs (CSV-based, targeting Neo4j) with additional nodes, relationships, and external metadata.

It is designed to extend the output produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG), adding properties sourced from external biomedical databases such as UMLS.

---

## Background

[pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG) is a companion CLI tool that downloads the full PubMed baseline and the NLM MeSH vocabulary and converts them into a set of Neo4j-compatible CSV files. The resulting graph covers PubMed articles together with their authors, journals, and citation links, as well as the MeSH vocabulary (Descriptors, Qualifiers, Supplemental records, and Concepts) and the indexing relationships between articles and MeSH headings.

---

## Installation

### With Nix

A Nix flake is provided. To build the package:

```sh
nix build github:c2fc2f/Extend-PubMed-MeSH-KG
```

To enter a development shell with all required tooling (Rust, rust-analyzer, clippy, rustfmt):

```sh
nix develop
```

### From source

```bash
git clone https://github.com/c2fc2f/Extend-PubMed-MeSH-KG
cd Extend-PubMed-MeSH-KG
cargo build --release
```

The compiled binary will be at `target/release/xpmkg`.

### With Cargo

```sh
cargo install --git https://github.com/c2fc2f/Extend-PubMed-MeSH-KG
```

---

## Usage

```
xpmkg <COMMAND>
```

Run `xpmkg --help` for the full list of available subcommands, or `xpmkg <COMMAND> --help` for subcommand-specific options.

---

## Subcommands

### `cui-umls`

Annotates MeSH nodes in the knowledge graph with their corresponding UMLS Concept Unique Identifiers (CUIs), sourced from the UMLS Metathesaurus.

This subcommand reads the `MRCONSO.RRF` file (from a local UMLS release) and appends a new `UMLSConceptsUI` column to the following CSV files produced by pm2kg:

- `MeSHConcept.csv`
- `MeSHDescriptor.csv`
- `MeSHQualifier.csv`
- `MeSHSupplemental.csv`

Each node is matched by its MeSH UI. When multiple CUIs map to the same UI, they are stored as a semicolon-separated list, compatible with Neo4j's array property import format (`string[]`).

**Arguments**

| Flag | Description |
|---|---|
| `-m`, `--mrconso <PATH>` | Path to the `MRCONSO.RRF` file from a UMLS release |
| `-o`, `--output <PATH>` | Path to the folder containing the pm2kg output CSV files |

**Example**

```sh
xpmkg cui-umls \
  --mrconso /path/to/umls/MRCONSO.RRF \
  --output /path/to/pm2kg/output/
```

The CSV files are updated in place. Each file is written to a temporary `.tmp` file first, then atomically swapped to replace the original only on success, so a partial failure does not corrupt existing data.

> **Prerequisites:** You need a UMLS license and a local UMLS release to obtain `MRCONSO.RRF`. You can request access at [https://www.nlm.nih.gov/research/umls/](https://www.nlm.nih.gov/research/umls/).

---

## Project structure

```
.
├── src/
│   ├── main.rs                        # CLI entry point (clap + Dispatch)
│   └── subcommand/
│       └── cui_umls/
│           ├── mod.rs                 # cui-umls subcommand logic
│           └── models.rs              # MRCONSO.RRF record schema
├── crates/
│   └── dispatch_derive/               # Internal proc-macro crate
│       └── src/lib.rs                 # #[derive(Dispatch)] implementation
├── nix/
│   └── package.nix                    # Nix package definition
├── flake.nix                          # Nix flake (build + devShell)
├── Cargo.toml
└── Cargo.lock
```

### `dispatch_derive`

An internal procedural macro crate that derives an async `dispatch` method on clap subcommand enums. It eliminates the boilerplate of hand-written `match` arms by generating dispatch from the enum variant's type path to the corresponding `run` function.

For example, a variant `CuiUmls(cui_umls::SubArgs)` automatically dispatches to `cui_umls::run(args).await`. See [`crates/dispatch_derive/README.md`](crates/dispatch_derive/README.md) for full documentation.

---

## License

MIT — see [LICENSE](LICENSE).
