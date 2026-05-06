# xpmkg kg-umls

Integrates the PubMed-MeSH knowledge graph with the UMLS knowledge graph by establishing cross-graph relationships between MeSH elements and UMLS atoms and concepts.

## Overview

This subcommand reads the `MRCONSO.RRF` file from a local UMLS release and produces two new Neo4j-compatible CSV relationship files that bridge the MeSH and UMLS ID spaces:

- **`MAPPED_TO.csv`** — links each MeSH UI to the UMLS concept (CUI) it maps to
- **`REFERENCE_OF.csv`** — links each UMLS atom (AUI) back to the MeSH UI it originates from

Both files are written from scratch to the specified output directory.

> **License requirement** — access to UMLS data requires a [UMLS Metathesaurus License](https://www.nlm.nih.gov/research/umls/index.html) from the NLM. The tool operates on a locally extracted UMLS release; it does not download data automatically.

## Requirements

- A Neo4j database that already contains:
  - The PubMed-MeSH graph produced by [pm2kg](https://github.com/c2fc2f/PubMed-MeSH-to-KG)
  - The UMLS graph produced by [umls2kg](https://github.com/c2fc2f/UMLS-to-KG)
- A locally extracted UMLS release containing `META/MRCONSO.RRF`

## Usage

```
xpmkg kg-umls --umls <PATH> [--output <PATH>]
```

| Flag | Short | Description | Default |
|---|---|---|---|
| `--umls <PATH>` | `-u` | Root directory of the extracted UMLS release (must contain `META/MRCONSO.RRF`) | *(required)* |
| `--output <PATH>` | `-o` | Directory where the output CSV files are written | `.` (current directory) |

### Example

```bash
xpmkg kg-umls \
  --umls /data/umls_2026 \
  --output /data/xpmkg-output/
```

## Output: Knowledge Graph Schema

Both files are formatted for [Neo4j's bulk CSV importer](https://neo4j.com/docs/operations-manual/current/tools/neo4j-admin/neo4j-admin-import/).

### Relationships

| File | Type | From → To | Description |
|---|---|---|---|
| `MAPPED_TO.csv` | `MAPPED_TO` | MeSH → UMLSConcept | Links a MeSH UI (Concept, Descriptor, Qualifier, or Supplemental) to its UMLS concept (CUI). When a UI maps to multiple CUIs, one row is written per CUI. |
| `REFERENCE_OF.csv` | `REFERENCE_OF` | UMLSAtom → MeSH | Links a UMLS atom (AUI) to the MeSH UI it references via its `SCUI` or `SDUI` field in `MRCONSO.RRF`. |

### ID spaces

| Label space | Used by |
|---|---|
| `MeSH` | MeSH nodes imported from pm2kg CSVs |
| `UMLSMetathesaurus` | UMLS concept and atom nodes imported from umls2kg CSVs |

## Importing into Neo4j

Once both `pm2kg`, `umls2kg`, and `xpmkg kg-umls` have finished writing their CSV files, use `neo4j-admin database import full` to bulk-load the combined graph into Neo4j. The command below assumes the pm2kg CSVs are in `../PubMed-MeSH/`, the umls2kg CSVs are in `../UMLS/`, and the xpmkg output CSVs are in the current directory.

> The database must be stopped before running an import. The `--overwrite-destination` flag will erase any existing data in the target database.

```bash
sudo JDK_JAVA_OPTIONS="--add-opens=java.base/java.nio=ALL-UNNAMED --add-opens=java.base/java.lang=ALL-UNNAMED" \
  neo4j-admin database import full neo4j \
    --verbose \
    --multiline-fields=true \
    --overwrite-destination \
    --skip-bad-relationships \
    --nodes=PubMed:PubMedArticle=../PubMed-MeSH/PubMedArticle.csv \
    --nodes=PubMed:PubMedAgent:PubMedCollective=../PubMed-MeSH/PubMedCollective.csv \
    --nodes=PubMed:PubMedAgent:PubMedPerson=../PubMed-MeSH/PubMedPerson.csv \
    --nodes=PubMed:PubMedJournal=../PubMed-MeSH/PubMedJournal.csv \
    --nodes=PubMed:PubMedKeyword=../PubMed-MeSH/PubMedKeyword.csv \
    --nodes=MeSH:MeSHDescriptor=../PubMed-MeSH/MeSHDescriptor.csv \
    --nodes=MeSH:MeSHQQualifier=../PubMed-MeSH/MeSHQualifier.csv \
    --nodes=MeSH:MeSHSupplemental=../PubMed-MeSH/MeSHSupplemental.csv \
    --nodes=MeSH:MeSHConcept=../PubMed-MeSH/MeSHConcept.csv \
    --nodes=MeSH:MeSHDescriptorQualified=../PubMed-MeSH/MeSHDescriptorQualified.csv \
    --relationships=HAS_AUTHOR=../PubMed-MeSH/HAS_AUTHOR.csv \
    --relationships=IS_PART_OF=../PubMed-MeSH/IS_PART_OF.csv \
    --relationships=HAS_KEYWORD=../PubMed-MeSH/HAS_KEYWORD.csv \
    --relationships=CITES=../PubMed-MeSH/CITES.csv \
    --relationships=NARROWER_THAN=../PubMed-MeSH/NARROWER_THAN.csv \
    --relationships=BROADER_THAN=../PubMed-MeSH/BROADER_THAN.csv \
    --relationships=RELATED_TO=../PubMed-MeSH/RELATED_TO.csv \
    --relationships=HAS_MESH=../PubMed-MeSH/HAS_MESH.csv \
    --relationships=HAS_SUPPLEMENTARY_MESH=../PubMed-MeSH/HAS_SUPPLEMENTARY_MESH.csv \
    --relationships=HAS_DESCRIPTOR=../PubMed-MeSH/HAS_DESCRIPTOR.csv \
    --relationships=HAS_QUALIFIER=../PubMed-MeSH/HAS_QUALIFIER.csv \
    --relationships=MAPPED_TO=../PubMed-MeSH/MAPPED_TO.csv \
    --relationships=HAS_PHARMACOLOGICAL_ACTION=../PubMed-MeSH/HAS_PHARMACOLOGICAL_ACTION.csv \
    --relationships=HAS_CONCEPT=../PubMed-MeSH/HAS_CONCEPT.csv \
    \
    --nodes=UMLS:UMLSMetathesaurus:UMLSConcept=../UMLS/UMLSConcept.csv \
    --nodes=UMLS:UMLSMetathesaurus:UMLSLexical=../UMLS/UMLSLexical.csv \
    --nodes=UMLS:UMLSMetathesaurus:UMLSString=../UMLS/UMLSString.csv \
    --nodes=UMLS:UMLSMetathesaurus:UMLSAtom=../UMLS/UMLSAtom.csv \
    --nodes=UMLS:UMLSMetathesaurus:UMLSAttribute:UMLSDefinition=../UMLS/UMLSDefinition.csv \
    --nodes=UMLS:UMLSSemanticNetwork:UMLSSemanticType=../UMLS/UMLSSemanticType.csv \
    --nodes=UMLS:UMLSSemanticNetwork:UMLSSemanticRelation=../UMLS/UMLSSemanticRelation.csv \
    --relationships=IS_ATOM_OF=../UMLS/IS_ATOM_OF.csv \
    --relationships=IS_STRING_OF=../UMLS/IS_STRING_OF.csv \
    --relationships=IS_LEXICAL_OF=../UMLS/IS_LEXICAL_OF.csv \
    --relationships=HAS_DEFINITION=../UMLS/HAS_DEFINITION.csv \
    --relationships=HAS_SEMANTIC_TYPE=../UMLS/HAS_SEMANTIC_TYPE.csv \
    --relationships=HAS_ALLOWED_QUALIFIER=../UMLS/HAS_ALLOWED_QUALIFIER.csv \
    --relationships=CHILD_OF=../UMLS/CHILD_OF.csv \
    --relationships=PARENT_OF=../UMLS/PARENT_OF.csv \
    --relationships=BROADER_THAN=../UMLS/BROADER_THAN.csv \
    --relationships=NARROWER_THAN=../UMLS/NARROWER_THAN.csv \
    --relationships=SYNONYM_OF=../UMLS/SYNONYM_OF.csv \
    --relationships=POSSIBLY_SYNONYM_OF=../UMLS/POSSIBLY_SYNONYM_OF.csv \
    --relationships=SIMILAR_TO=../UMLS/SIMILAR_TO.csv \
    --relationships=RELATED_TO=../UMLS/RELATED_TO.csv \
    --relationships=HAS_OTHER_RELATIONSHIP=../UMLS/HAS_OTHER_RELATIONSHIP.csv \
    --relationships=QUALIFIED_BY=../UMLS/QUALIFIED_BY.csv \
    --relationships=NOT_RELATED_TO=../UMLS/NOT_RELATED_TO.csv \
    --relationships=DELETED=../UMLS/DELETED.csv \
    --relationships=UNASSIGNED=../UMLS/UNASSIGNED.csv \
    --relationships=IS_A=../UMLS/SN-IS_A.csv \
    --relationships=PART_OF=../UMLS/SN-PART_OF.csv \
    --relationships=CONCEPTUAL_PART_OF=../UMLS/SN-CONCEPTUAL_PART_OF.csv \
    --relationships=PHYSICALLY_RELATED_TO=../UMLS/SN-PHYSICALLY_RELATED_TO.csv \
    --relationships=SPATIALLY_RELATED_TO=../UMLS/SN-SPATIALLY_RELATED_TO.csv \
    --relationships=TEMPORALLY_RELATED_TO=../UMLS/SN-TEMPORALLY_RELATED_TO.csv \
    --relationships=FUNCTIONALLY_RELATED_TO=../UMLS/SN-FUNCTIONALLY_RELATED_TO.csv \
    --relationships=CONCEPTUALLY_RELATED_TO=../UMLS/SN-CONCEPTUALLY_RELATED_TO.csv \
    --relationships=ASSOCIATED_WITH=../UMLS/SN-ASSOCIATED_WITH.csv \
    --relationships=CAUSES=../UMLS/SN-CAUSES.csv \
    --relationships=PRODUCES=../UMLS/SN-PRODUCES.csv \
    --relationships=AFFECTS=../UMLS/SN-AFFECTS.csv \
    --relationships=DISRUPTS=../UMLS/SN-DISRUPTS.csv \
    --relationships=PREVENTS=../UMLS/SN-PREVENTS.csv \
    --relationships=TREATS=../UMLS/SN-TREATS.csv \
    --relationships=MANAGES=../UMLS/SN-MANAGES.csv \
    --relationships=COMPLICATES=../UMLS/SN-COMPLICATES.csv \
    --relationships=MANIFESTATION_OF=../UMLS/SN-MANIFESTATION_OF.csv \
    --relationships=DIAGNOSES=../UMLS/SN-DIAGNOSES.csv \
    --relationships=INDICATES=../UMLS/SN-INDICATES.csv \
    --relationships=ASSESSES_EFFECT_OF=../UMLS/SN-ASSESSES_EFFECT_OF.csv \
    --relationships=MEASURES=../UMLS/SN-MEASURES.csv \
    --relationships=MEASUREMENT_OF=../UMLS/SN-MEASUREMENT_OF.csv \
    --relationships=EVALUATION_OF=../UMLS/SN-EVALUATION_OF.csv \
    --relationships=PERFORMS=../UMLS/SN-PERFORMS.csv \
    --relationships=CARRIES_OUT=../UMLS/SN-CARRIES_OUT.csv \
    --relationships=PRACTICES=../UMLS/SN-PRACTICES.csv \
    --relationships=USES=../UMLS/SN-USES.csv \
    --relationships=PROCESS_OF=../UMLS/SN-PROCESS_OF.csv \
    --relationships=RESULT_OF=../UMLS/SN-RESULT_OF.csv \
    --relationships=BRINGS_ABOUT=../UMLS/SN-BRINGS_ABOUT.csv \
    --relationships=OCCURS_IN=../UMLS/SN-OCCURS_IN.csv \
    --relationships=EXHIBITS=../UMLS/SN-EXHIBITS.csv \
    --relationships=INTERACTS_WITH=../UMLS/SN-INTERACTS_WITH.csv \
    --relationships=CO_OCCURS_WITH=../UMLS/SN-CO_OCCURS_WITH.csv \
    --relationships=PRECEDES=../UMLS/SN-PRECEDES.csv \
    --relationships=LOCATION_OF=../UMLS/SN-LOCATION_OF.csv \
    --relationships=CONTAINS=../UMLS/SN-CONTAINS.csv \
    --relationships=CONNECTED_TO=../UMLS/SN-CONNECTED_TO.csv \
    --relationships=ADJACENT_TO=../UMLS/SN-ADJACENT_TO.csv \
    --relationships=INTERCONNECTS=../UMLS/SN-INTERCONNECTS.csv \
    --relationships=SURROUNDS=../UMLS/SN-SURROUNDS.csv \
    --relationships=TRAVERSES=../UMLS/SN-TRAVERSES.csv \
    --relationships=BRANCH_OF=../UMLS/SN-BRANCH_OF.csv \
    --relationships=TRIBUTARY_OF=../UMLS/SN-TRIBUTARY_OF.csv \
    --relationships=CONSISTS_OF=../UMLS/SN-CONSISTS_OF.csv \
    --relationships=INGREDIENT_OF=../UMLS/SN-INGREDIENT_OF.csv \
    --relationships=DERIVATIVE_OF=../UMLS/SN-DERIVATIVE_OF.csv \
    --relationships=DEVELOPMENTAL_FORM_OF=../UMLS/SN-DEVELOPMENTAL_FORM_OF.csv \
    --relationships=DEGREE_OF=../UMLS/SN-DEGREE_OF.csv \
    --relationships=PROPERTY_OF=../UMLS/SN-PROPERTY_OF.csv \
    --relationships=METHOD_OF=../UMLS/SN-METHOD_OF.csv \
    --relationships=ANALYZES=../UMLS/SN-ANALYZES.csv \
    --relationships=ISSUE_IN=../UMLS/SN-ISSUE_IN.csv \
    \
    --relationships=MAPPED_TO=./MAPPED_TO.csv \
    --relationships=REFERENCE_OF=./REFERENCE_OF.csv \
    --additional-config=/var/lib/neo4j/conf/neo4j.conf
```

The two `--add-opens` JVM flags are required on recent JDK versions to allow Neo4j's importer to access internal NIO and language APIs. Adjust `--additional-config` to point to your actual `neo4j.conf` if it lives elsewhere.
