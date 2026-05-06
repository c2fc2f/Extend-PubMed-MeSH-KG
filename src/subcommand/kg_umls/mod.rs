//! Integrates the PubMed-MeSH knowledge graph with the UMLS Knowledge Graph
//! by establishing relationships between MeSH elements and UMLS Atoms and
//! Concepts

use clap::Args;
use csv_async::AsyncWriter;
use futures::StreamExt;
use tokio::fs::File;
use umls::{UMLS, metathesaurus::conso::models::CoNSoRecord};

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::ExitCode,
    rc::Rc,
};

/// Integrates the PubMed-MeSH knowledge graph with the UMLS Knowledge Graph
/// by establishing relationships between MeSH elements and UMLS Atoms and
/// Concepts
#[derive(Args)]
pub struct SubArgs {
    /// Directory containing the UMLS Metathesaurus `META/MRCONSO.RRF` file
    #[arg(short, long)]
    umls: PathBuf,

    /// Directory where CSV files are written
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
}

/// Entry point to this command
pub async fn run(args: SubArgs) -> ExitCode {
    let umls: UMLS = UMLS::new(args.umls);

    let mut cuis: HashMap<Rc<String>, HashSet<String>> = HashMap::default();

    let mut reference_of: AsyncWriter<File> = AsyncWriter::from_writer(
        match File::create(args.output.join("REFERENCE_OF.csv")).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error during creation of the CSV files:\n{e:?}");
                return ExitCode::FAILURE;
            }
        },
    );

    if let Err(e) = reference_of
        .write_record([":START_ID(UMLSMetathesaurus)", ":END_ID(MeSH)"])
        .await
    {
        eprintln!("Error during writing of the CSV files:\n{:?}", e);
        return ExitCode::FAILURE;
    }

    let mut mapped_to: AsyncWriter<File> = AsyncWriter::from_writer(
        match File::create(args.output.join("MAPPED_TO.csv")).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error during creation of the CSV files:\n{e:?}");
                return ExitCode::FAILURE;
            }
        },
    );

    if let Err(e) = mapped_to
        .write_record([":START_ID(MeSH)", ":END_ID(UMLSMetathesaurus)"])
        .await
    {
        eprintln!("Error during writing of the CSV files:\n{:?}", e);
        return ExitCode::FAILURE;
    }

    let mut stream = umls.concept_names_and_sources();

    while let Some(record) = stream.next().await {
        let record: CoNSoRecord = match record {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Deserialization error: {e:?}");
                return ExitCode::FAILURE;
            }
        };

        if record.sab != "MSH" {
            continue;
        }

        let r: std::io::Result<()> = async {
            let cui: Rc<String> = Rc::new(record.cui);
            let set: &mut HashSet<String> = cuis.entry(Rc::clone(&cui)).or_default();

            if let Some(scui) = record.scui {
                reference_of.write_record([&record.aui, &scui]).await?;
                if !set.contains(&scui) {
                    mapped_to.write_record([&scui, cui.as_str()]).await?;
                    set.insert(scui);
                }
            }
            if let Some(sdui) = record.sdui {
                reference_of.write_record([&record.aui, &sdui]).await?;
                if !set.contains(&sdui) {
                    mapped_to.write_record([&sdui, cui.as_str()]).await?;
                    set.insert(sdui);
                }
            }

            Ok(())
        }
        .await;

        if let Err(e) = r {
            eprintln!("Error during writing of the CSV files:\n{:?}", e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
