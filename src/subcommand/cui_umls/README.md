# xpmkg cui-umls

Annotates MeSH nodes in the knowledge graph with their corresponding UMLS Concept Unique Identifiers (CUIs), sourced from the UMLS Metathesaurus.

## Overview

This subcommand reads the `MRCONSO.RRF` file from a local UMLS release, builds a mapping from every MeSH UI to the set of CUIs that reference it, and appends a new `UMLSConceptsUI` column to the following CSV files produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG):

- `MeSHConcept.csv`
- `MeSHDescriptor.csv`
- `MeSHQualifier.csv`
- `MeSHSupplemental.csv`

Each node is matched by its MeSH UI. When multiple CUIs map to the same UI, they are stored as a semicolon-separated list, compatible with Neo4j's array property import format (`string[]`).

To ensure data integrity, each file is first written to a `.tmp` sibling, then atomically renamed to replace the original only on success. A partial failure therefore never corrupts existing data.

> **License requirement** — access to UMLS data requires a [UMLS Metathesaurus License](https://www.nlm.nih.gov/research/umls/index.html) from the NLM. The tool operates on a locally extracted UMLS release; it does not download data automatically.

## Requirements

- Output CSVs produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG)
- A locally extracted UMLS release containing `META/MRCONSO.RRF`

## Usage

```
xpmkg cui-umls --umls <PATH> --output <PATH>
```

| Flag | Short | Description |
|---|---|---|
| `--umls <PATH>` | `-u` | Root directory of the extracted UMLS release (must contain `META/MRCONSO.RRF`) |
| `--output <PATH>` | `-o` | Directory containing the pm2kg output CSV files to be annotated |

### Example

```bash
xpmkg cui-umls \
  --umls /data/umls_2026 \
  --output /data/pm2kg-output/
```

## Output

The four MeSH CSV files are updated **in place**. A new column is appended to each:

| Column | Neo4j type | Description |
|---|---|---|
| `UMLSConceptsUI:string[]` | `string[]` | Semicolon-separated list of UMLS CUIs mapping to this MeSH UI |

Nodes with no CUI mapping receive an empty string for that field.

### Example — before / after

**Before (`MeSHDescriptor.csv`, partial):**

```csv
:ID(MeSH),name,...
D000001,Calcimycin,...
```

**After:**

```csv
:ID(MeSH),name,...,UMLSConceptsUI:string[]
D000001,Calcimycin,...,C0006611
```
