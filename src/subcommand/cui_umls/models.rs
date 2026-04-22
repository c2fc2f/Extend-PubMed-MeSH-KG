//! Module regrouping datatype

use serde::{Deserialize, Deserializer, Serialize};

/// Represents a single record from the MRCONSO.RRF file (Concept Names and
/// Sources). This struct defines the schema for the core Metathesaurus
/// concept information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct MrConsoRecord {
    /// Unique identifier for concept (CUI)
    pub cui: String,

    /// Language of term (LAT)
    pub lat: String,

    /// Term status (TS)
    pub ts: String,

    /// Unique identifier for term (LUI)
    pub lui: String,

    /// String type (STT)
    pub stt: String,

    /// Unique identifier for string (SUI)
    pub sui: String,

    /// Atom status - preferred (Y) or not (N) for this string within this
    /// concept (ISPREF)
    #[serde(rename = "ISPREF", deserialize_with = "deserialize_yes_no")]
    pub is_pref: bool,

    /// Unique identifier for atom - variable length field, 8 or 9 characters
    /// (AUI)
    pub aui: String,

    /// Source asserted atom identifier [optional] (SAUI)
    pub saui: Option<String>,

    /// Source asserted concept identifier [optional] (SCUI)
    pub scui: Option<String>,

    /// Source asserted descriptor identifier [optional] (SDUI)
    pub sdui: Option<String>,

    /// Abbreviated source name (SAB). Max length 20 alphanumeric characters.
    pub sab: String,

    /// Abbreviation for term type in source vocabulary (e.g., PN, CD) (TTY)
    pub tty: String,

    /// Most useful source asserted identifier (CODE)
    pub code: String,

    /// The actual string/term (STR)
    pub str: String,

    /// Source restriction level (SRL)
    pub srl: String,

    /// Suppressible flag. Values: O (obsolete), E (editor marked),
    /// Y (suppressible), N (none) (SUPPRESS)
    pub suppress: char,

    /// Content View Flag. Bit field used to flag rows included in Content
    /// View (CVF)
    pub cvf: Option<String>,
}

/// Custom deserializer to convert UMLS 'Y'/'N' flags into a boolean.
///
/// # Arguments
/// * `deserializer` - The Serde deserializer instance.
///
/// # Returns
/// * `Ok(true)` if the input character is 'Y'.
/// * `Ok(false)` for any other character (typically 'N').
/// * `Err` if the input cannot be deserialized as a character.
fn deserialize_yes_no<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(char::deserialize(deserializer)? == 'Y')
}
