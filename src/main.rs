//! A multitool for extending PubMed-MeSH knowledge graphs (CSV-based for
//! Neo4J) with additional nodes, relationships, and external metadata

mod subcommand;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use dispatch_derive::Dispatch;

use crate::subcommand::cui_umls;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// A multitool for extending PubMed-MeSH knowledge graphs (CSV-based for
/// Neo4J) with additional nodes, relationships, and external metadata
pub struct Cli {
    /// The specific operation to perform on the knowledge graph
    #[command(subcommand)]
    command: Command,
}

/// List of available subcommands for graph manipulation and metadata
/// ingestion
#[derive(Subcommand, Dispatch)]
enum Command {
    /// Command to add UMLS CUI properties to every node of MeSH Descriptor
    /// and MeSH Concept
    CuiUmls(cui_umls::SubArgs),
}

#[tokio::main]
async fn main() -> ExitCode {
    Cli::parse().command.dispatch().await
}
