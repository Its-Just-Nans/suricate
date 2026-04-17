//! Gedcom parsing

pub(crate) mod parser;
pub(crate) mod tokenizer;
pub(crate) mod types;

use serde::{Deserialize, Serialize};
use types::{Family, Header, Individual, Media, Repository, Source, Submitter};

#[derive(Debug, Default, Serialize, Deserialize)]
/// The data structure representing all the data within a gedcom file
pub struct GedcomData {
    /// Header containing file metadata
    pub header: Header,
    /// List of submitters of the facts
    pub submitters: Vec<Submitter>,
    /// Individuals within the family tree
    pub individuals: Vec<Individual>,
    /// The family units of the tree, representing relationships between individuals
    pub families: Vec<Family>,
    /// A data repository where `sources` are held
    pub repositories: Vec<Repository>,
    /// Sources of facts. _ie._ book, document, census, etc.
    pub sources: Vec<Source>,
    /// A multimedia asset linked to a fact
    pub multimedia: Vec<Media>,
}
