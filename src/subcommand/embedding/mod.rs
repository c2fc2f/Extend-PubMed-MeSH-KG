//! Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and
//! Supplementals) and PubMed entities (Articles and Keywords) with an
//! additional "embedding" property

use clap::{Args, ValueEnum};
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder, StringRecord};
use futures::StreamExt;
use rig_core::{
  client::EmbeddingsClient,
  embeddings::{Embedding, EmbeddingError, EmbeddingModel},
  providers::{ollama, openai},
};
use tokio::fs::{File, remove_file, rename};

use std::{io, path::PathBuf, process::ExitCode, sync::Arc};

/// Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and
/// Supplementals) and PubMed entities (Articles and Keywords) with an
/// additional "embedding" property
#[derive(Args)]
pub struct SubArgs {
  /// The API provider to use for generating the embeddings
  #[arg(short, long)]
  provider: Provider,

  /// An optional custom base URL for the provider's API endpoint
  #[arg(long)]
  base_url: Option<String>,

  /// The authentication key for the selected provider
  #[arg(long)]
  api_key: Option<String>,

  /// The specific model to use for the embeddings
  #[arg(short, long)]
  model: String,

  /// Skips the enrichment of PubMed entities (Articles and Keywords)
  #[arg(short, long, action)]
  no_pubmed: bool,

  /// Path to the result folder of pm2kg
  #[arg(short, long)]
  output: PathBuf,
}

/// Supported API providers for generating text embeddings
#[derive(Clone, ValueEnum)]
enum Provider {
  /// OpenAI's remote embedding API
  OpenAI,
  /// A Ollama instance
  Ollama,
}

/// A dynamic dispatcher for text embedding models from various LLM providers
pub enum DynamicEmbeder {
  /// Wraps an [`ollama::EmbeddingModel`].
  Ollama(ollama::EmbeddingModel),

  /// Wraps an [`openai::EmbeddingModel`].
  OpenAI(openai::EmbeddingModel),
}

impl DynamicEmbeder {
  /// Generates an embedding for the provided text by delegating the request
  /// to the currently active LLM provider.
  ///
  /// # Arguments
  ///
  /// * `text` - A string slice containing the content to be embedded.
  ///
  /// # Errors
  ///
  /// Returns an [`EmbeddingError`] if the underlying provider encounters an
  /// issue. This can include network timeouts, authentication failures
  /// (e.g., an invalid API key), or API rate limits.
  pub async fn embed_text(
    &self,
    text: &str,
  ) -> Result<Embedding, EmbeddingError> {
    match self {
      Self::Ollama(a) => a.embed_text(text).await,
      Self::OpenAI(a) => a.embed_text(text).await,
    }
  }
}

/// Entry point to this command
pub async fn run(args: SubArgs) -> ExitCode {
  let agent = match match args.provider {
    Provider::Ollama => {
      let mut client = ollama::Client::builder()
        .api_key(args.api_key.as_deref().unwrap_or_default());
      if let Some(base_url) = args.base_url {
        client = client.base_url(base_url);
      }
      client
        .build()
        .map(|c| DynamicEmbeder::Ollama(c.embedding_model(args.model)))
    }
    Provider::OpenAI => {
      let Some(api_key) = args.api_key else {
        eprintln!(
          "Error: Missing API key. An API key is strictly required when using the OpenAI provider."
        );
        return ExitCode::FAILURE;
      };
      let mut client = openai::Client::builder().api_key(api_key);
      if let Some(base_url) = args.base_url {
        client = client.base_url(base_url);
      }
      client
        .build()
        .map(|c| DynamicEmbeder::OpenAI(c.embedding_model(args.model)))
    }
  } {
    Ok(a) => Arc::new(a),
    Err(e) => {
      eprintln!("Error during creation of the CSV files:\n{e:?}");
      return ExitCode::FAILURE;
    }
  };

  let join: std::io::Result<((), (), (), (), (), ())> = tokio::try_join!(
    add_embedding(
      args.output.join("MeSHConcept.csv"),
      [1, 2],
      Arc::clone(&agent)
    ),
    add_embedding(
      args.output.join("MeSHDescriptor.csv"),
      [1],
      Arc::clone(&agent)
    ),
    add_embedding(
      args.output.join("MeSHQualifier.csv"),
      [1],
      Arc::clone(&agent)
    ),
    add_embedding(
      args.output.join("MeSHSupplemental.csv"),
      [2],
      Arc::clone(&agent)
    ),
    async {
      if !args.no_pubmed {
        add_embedding(
          args.output.join("PubMedArticle.csv"),
          [1, 2],
          Arc::clone(&agent),
        )
        .await
      } else {
        Ok(())
      }
    },
    async {
      if !args.no_pubmed {
        add_embedding(
          args.output.join("PubMedKeyword.csv"),
          [1],
          Arc::clone(&agent),
        )
        .await
      } else {
        Ok(())
      }
    },
  );

  if let Err(e) = join {
    eprintln!("Error during writing of the CSV files:\n{:?}", e);
    return ExitCode::FAILURE;
  }

  ExitCode::SUCCESS
}

/// Appends a new column containing text embeddings to a CSV file.
///
/// This function reads an existing CSV file asynchronously, adds a new header
/// `"embedding:float[]"`, and appends a semicolon-separated list of floats
/// representing the embedding vector to each row.
///
/// The text content used to generate the embedding is constructed by
/// concatenating the columns specified by the `fields` array for each row.
///
/// To ensure data integrity, the operation is performed using a temporary
/// file (with a `.tmp` extension), which safely replaces the original file
/// only upon successful completion of all API calls and disk writes.
///
/// # Arguments
///
/// - `file` - A `PathBuf` representing the path to the target CSV file.
/// - `fields` - An array of indices (`[usize; N]`) specifying which columns
///   should be extracted and concatenated to form the input for the embedder.
/// - `embeder` - An `Arc<DynamicEmbeder>` used to call the LLM provider and
///   generate the embedding vectors.
///
/// # Errors
///
/// This function will return an [`std::io::Result::Err`] in the following
/// situations:
/// - Standard I/O errors occur during file opening, creation, writing,
///   flushing, or renaming.
/// - The input CSV file is completely empty, lacking even a header row
///   ([`std::io::ErrorKind::InvalidData`]).
/// - A record in the CSV file is missing one of the columns requested in the
///   `fields` array ([`std::io::ErrorKind::InvalidData`]).
/// - The underlying LLM embedder fails to generate an embedding for a given
///   row's text ([`std::io::ErrorKind::Other`]).
async fn add_embedding<const N: usize>(
  file: PathBuf,
  fields: [usize; N],
  embeder: Arc<DynamicEmbeder>,
) -> io::Result<()> {
  let tmp: PathBuf = file.with_added_extension("tmp");

  let mut rdr = AsyncReaderBuilder::new()
    .delimiter(b',')
    .has_headers(false)
    .create_reader(File::open(&file).await?);

  let mut wri = AsyncWriterBuilder::new()
    .delimiter(b',')
    .has_headers(false)
    .create_writer(File::create(&tmp).await?);

  let mut records = rdr.records();

  if let Some(record_result) = records.next().await {
    let mut first = record_result?;
    first.push_field("embedding:float[]");
    wri.write_record(&first).await?;
  } else {
    remove_file(&tmp).await?;
    return Err(io::Error::new(
      io::ErrorKind::InvalidData,
      "The source CSV file is completely empty (no headers found).",
    ));
  }

  while let Some(record_result) = records.next().await {
    let mut record: StringRecord = record_result?;

    let content = fields.iter().try_fold(String::new(), |mut acc, idx| {
      let field_value = record.get(*idx).ok_or_else(|| {
        io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Missing expected CSV field at index {idx}."),
        )
      })?;

      acc.push_str(field_value);
      acc.push('\n');

      io::Result::Ok(acc)
    })?;

    let embedding = embeder.embed_text(&content).await.map_err(|e| {
      io::Error::other(format!(
        "Failed to generate embedding from the LLM provider: {e:?}"
      ))
    })?;

    record.push_field(
      &embedding
        .vec
        .iter()
        .map(|val| val.to_string())
        .collect::<Vec<String>>()
        .join(";"),
    );
    wri.write_record(&record).await?;
  }

  wri.flush().await?;

  remove_file(&file).await?;
  rename(tmp, file).await?;

  Ok(())
}
