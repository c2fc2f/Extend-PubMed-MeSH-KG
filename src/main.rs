//! A multitool for extending PubMed-MeSH knowledge graphs (CSV-based for
//! Neo4J) with additional nodes, relationships, and external metadata

mod subcommand;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use dispatch_derive::Dispatch;

use crate::subcommand::{cui_umls, kg_umls};

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
    /// Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and
    /// Supplementals) with corresponding UMLS Concept Unique Identifiers
    /// (CUIs)
    CuiUmls(cui_umls::SubArgs),
    /// Integrates the PubMed-MeSH knowledge graph with the UMLS Knowledge
    /// Graph by establishing relationships between MeSH elements and UMLS
    /// Atoms and Concepts
    KgUmls(kg_umls::SubArgs),
}

#[tokio::main]
async fn main() -> ExitCode {
    Cli::parse().command.dispatch().await
}
