# xpmkg embedding

Enriches MeSH entities (Concepts, Descriptors, Qualifiers, and Supplementals) and PubMed entities (Articles and Keywords) with a dense vector representation by calling a remote or local embedding model.

## Overview

This subcommand reads the CSV files produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG), sends the textual content of each entity to an embedding API, and appends a new `embedding:float[]` column to each of the following files:

- `MeSHConcept.csv`
- `MeSHDescriptor.csv`
- `MeSHQualifier.csv`
- `MeSHSupplemental.csv`
- `PubMedArticle.csv` *(unless `--no-pubmed` is set)*
- `PubMedKeyword.csv` *(unless `--no-pubmed` is set)*

The embedding vector is stored as a semicolon-separated list of floats, compatible with Neo4j's array property import format (`float[]`). Two providers are currently supported: **OpenAI** (and any OpenAI-compatible remote endpoint) and **Ollama**.

All files are processed concurrently. To ensure data integrity, each file is first written to a `.tmp` sibling, then atomically renamed to replace the original only on success. A partial failure therefore never corrupts existing data.

> **Note** — the PubMed entity files (`PubMedArticle.csv`, `PubMedKeyword.csv`) can be excluded from enrichment with the `--no-pubmed` flag, which can significantly reduce the number of API calls when only MeSH annotation is needed.

## Requirements

- Output CSVs produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG)
- Access to one of the supported embedding providers:
  - **OpenAI**: a valid API key with access to an embedding model
  - **Ollama**: a running [Ollama](https://ollama.com) instance with the target model already pulled (`ollama pull <model>`)

## Usage

```
xpmkg embedding --provider <PROVIDER> --model <MODEL> --output <PATH> [OPTIONS]
```

| Flag | Short | Description | Default |
|---|---|---|---|
| `--provider <PROVIDER>` | `-p` | API provider to use (`openai` or `ollama`) | *(required)* |
| `--model <MODEL>` | `-m` | Embedding model identifier for the selected provider | *(required)* |
| `--output <PATH>` | `-o` | Directory containing the pm2kg output CSV files to be enriched | *(required)* |
| `--api-key <KEY>` | | Authentication key for the provider (strictly required for OpenAI) | — |
| `--base-url <URL>` | | Custom base URL for the provider's API endpoint | *(provider default)* |
| `--no-pubmed` | `-n` | Skip enrichment of PubMed entities (`PubMedArticle.csv`, `PubMedKeyword.csv`) | false |

### Examples

**With OpenAI:**

```bash
xpmkg embedding \
  --provider openai \
  --api-key sk-... \
  --model text-embedding-3-small \
  --output /data/pm2kg-output/
```

**With a local Ollama instance:**

```bash
xpmkg embedding \
  --provider ollama \
  --model embeddinggemma:latest \
  --output /data/pm2kg-output/
```

**With a custom OpenAI-compatible endpoint, MeSH entities only:**

```bash
xpmkg embedding \
  --provider openai \
  --base-url http://localhost:11434/v1 \
  --api-key local \
  --model my-embed-model \
  --no-pubmed \
  --output /data/pm2kg-output/
```

## Providers

### OpenAI

An API key is strictly required; the subcommand will exit immediately with an error if it is missing. The model identifier must match one of the available [OpenAI embedding models](https://platform.openai.com/docs/guides/embeddings) (e.g. `text-embedding-3-small`, `text-embedding-3-large`, `text-embedding-ada-002`).

A custom `--base-url` can be used to target any OpenAI-compatible endpoint (e.g. a self-hosted inference server or a third-party gateway).

### Ollama

No API key is required. The model must already be available locally before running the subcommand:

```bash
ollama pull embeddinggemma:latest
```

The default base URL is `http://localhost:11434`; use `--base-url` to target a remote or non-default Ollama instance.

## Output

A new column is appended **in place** to each processed file:

| Column | Neo4j type | Description |
|---|---|---|
| `embedding:float[]` | `float[]` | Semicolon-separated list of floats representing the embedding vector of the entity |

### Example — before / after

**Before (`MeSHDescriptor.csv`, partial):**

```csv
:ID(MeSH),name,...
D000001,Calcimycin,...
```

**After:**

```csv
:ID(MeSH),name,...,embedding:float[]
D000001,Calcimycin,...,0.0213;-0.0437;0.1172
```
