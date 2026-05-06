//! Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and
//! Supplementals) with corresponding UMLS Concept Unique Identifiers (CUIs)

use clap::Args;
use csv_async::{
    AsyncReader, AsyncReaderBuilder, AsyncWriter, AsyncWriterBuilder, StringRecord,
    StringRecordsStream,
};
use futures::StreamExt;
use tokio::fs::{File, remove_file, rename};
use umls::{UMLS, metathesaurus::conso::models::CoNSoRecord};

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::ExitCode,
    rc::Rc,
};

/// Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and
/// Supplementals) with corresponding UMLS Concept Unique Identifiers (CUIs)
#[derive(Args)]
pub struct SubArgs {
    /// Directory containing the UMLS Metathesaurus `META/MRCONSO.RRF` file
    #[arg(short, long)]
    umls: PathBuf,

    /// Path to the result folder of pm2kg
    #[arg(short, long)]
    output: PathBuf,
}

/// Entry point to this command
pub async fn run(args: SubArgs) -> ExitCode {
    let umls: UMLS = UMLS::new(args.umls);
    let mut ui_to_cui: HashMap<String, HashSet<Rc<String>>> = HashMap::default();

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

        let cui: Rc<String> = Rc::new(record.cui);

        if let Some(scui) = record.scui {
            let cui: Rc<String> = Rc::clone(&cui);
            match ui_to_cui.get_mut(&scui) {
                None => {
                    ui_to_cui.insert(scui, HashSet::from([cui]));
                }
                Some(v) => {
                    v.insert(cui);
                }
            }
        }

        if let Some(sdui) = record.sdui {
            match ui_to_cui.get_mut(&sdui) {
                None => {
                    ui_to_cui.insert(sdui, HashSet::from([cui]));
                }
                Some(v) => {
                    v.insert(cui);
                }
            }
        }
    }

    let join: std::io::Result<((), (), (), ())> = tokio::try_join!(
        add_cui(args.output.join("MeSHConcept.csv"), &ui_to_cui),
        add_cui(args.output.join("MeSHDescriptor.csv"), &ui_to_cui),
        add_cui(args.output.join("MeSHQualifier.csv"), &ui_to_cui),
        add_cui(args.output.join("MeSHSupplemental.csv"), &ui_to_cui),
    );

    if let Err(e) = join {
        eprintln!("Error during writing of the CSV files:\n{:?}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Appends a new column containing UMLS Concept Unique Identifiers (CUIs) to
/// a CSV file.
///
/// This function reads an existing CSV file asynchronously, adds a new header
/// `"UMLSConceptsUI"`, and appends a semicolon-separated list of CUIs to each
/// row. The CUIs are retrieved from the provided `ui_to_cui` mapping, using
/// the first column of each row as the lookup key.
///
/// To ensure data integrity, the operation is performed using a temporary
/// file (with a `.tmp` extension), which safely replaces the original file
/// only upon successful completion of all writes.
///
/// # Arguments
///
/// - `file` - A `PathBuf` representing the path to the target CSV file.
/// - `ui_to_cui` - A reference to a [`HashMap`] where the key is a UI and the
///   value is a [`HashSet`] of reference-counted strings representing the
///   associated CUIs.
///
/// # Errors
///
/// This function will return an [`std::io::Result::Err`] in the following
/// situations:
/// - Standard I/O errors occur during file opening, creation, writing,
///   flushing, or renaming.
/// - The input CSV file is completely empty
///   ([`std::io::ErrorKind::InvalidData`]).
/// - A record in the CSV file is missing the first column
///   ([`std::io::ErrorKind::InvalidData`]).
async fn add_cui(
    file: PathBuf,
    ui_to_cui: &HashMap<String, HashSet<Rc<String>>>,
) -> std::io::Result<()> {
    let tmp: PathBuf = file.with_added_extension("tmp");

    let mut rdr: AsyncReader<File> = AsyncReaderBuilder::new()
        .delimiter(b',')
        .has_headers(false)
        .create_reader(File::open(&file).await?);

    let mut wri: AsyncWriter<File> = AsyncWriterBuilder::new()
        .delimiter(b',')
        .has_headers(false)
        .create_writer(File::create(&tmp).await?);

    let mut records: StringRecordsStream<File> = rdr.records();

    if let Some(record_result) = records.next().await {
        let mut first: StringRecord = record_result?;
        first.push_field("UMLSConceptsUI:string[]");
        wri.write_record(&first).await?;
    } else {
        remove_file(&tmp).await?;
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "The file doesn't respect the expected format: file is empty.",
        ));
    }

    while let Some(record_result) = records.next().await {
        let mut record: StringRecord = record_result?;

        let ui: &str = record.get(0).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "The file doesn't respect the expected format: a record is missing the first column.",
        ))?;

        record.push_field(
            ui_to_cui
                .get(ui)
                .map(|cuis| {
                    cuis.iter()
                        .map(|rc| rc.as_str())
                        .collect::<Vec<&str>>()
                        .join(";")
                })
                .as_deref()
                .unwrap_or(""),
        );
        wri.write_record(&record).await?;
    }

    wri.flush().await?;

    remove_file(&file).await?;
    rename(tmp, file).await?;

    Ok(())
}
