# dispatch-derive

A procedural macro that derives an async `dispatch` method on
[clap](https://docs.rs/clap) subcommand enums.

Instead of writing a `match` arm for every subcommand by hand, annotate
your enum and let the macro generate the delegation for you.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
dispatch-derive = { path = "../dispatch-derive" }
clap = { version = "4", features = ["derive"] }
```

Define a `run` function in each subcommand module:

```rust
// src/stats/mod.rs
use clap::Args;
use std::process::ExitCode;

#[derive(Args)]
pub struct SubArgs {
    #[arg(long)]
    pub verbose: bool,
}

pub async fn run(args: SubArgs) -> ExitCode {
    ExitCode::SUCCESS
}
```

Derive `Dispatch` alongside `clap::Subcommand`:

```rust
use clap::Parser;
use dispatch_derive::Dispatch;

mod stats;
mod report;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Dispatch)]
enum Command {
    Stats(stats::SubArgs),
    Report(report::SubArgs),
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    Cli::parse().command.dispatch().await
}
```

The macro expands `Command` to:

```rust
impl Command {
    pub async fn dispatch(self) -> std::process::ExitCode {
        match self {
            Command::Stats(args)  => stats::run(args).await,
            Command::Report(args) => report::run(args).await,
        }
    }
}
```

## Convention

| What you write | What gets called |
|---|---|
| `Stats(stats::SubArgs)` | `stats::run(args).await` |
| `Report(report::SubArgs)` | `report::run(args).await` |
| `Db(db::SubArgs)` | `db::run(args).await` |

The last segment of the field's type path is stripped and replaced with
`::run`. Everything before it is treated as the module path.

## Requirements

- Each variant must be a **single-field tuple**: `Foo(foo::SubArgs)`
- The field type must be a **plain path** (no references or generics)
- Each module must expose `pub async fn run(args: SubArgs) -> ExitCode`

All violations are reported as **compile-time errors**.

## License

MIT
