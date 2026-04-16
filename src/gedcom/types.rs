use serde::{Deserialize, Serialize};

type Xref = String;

/// Physical address at which a fact occurs
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Address {
    pub value: Option<String>,
    pub adr1: Option<String>,
    pub adr2: Option<String>,
    pub adr3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub post: Option<String>,
    pub country: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    Adoption,
    Birth,
    Burial,
    Death,
    Christening,
    Marriage,
    Residence,
    SourceData(String),

    // "Other" is used to construct an event without requiring an explicit event type
    #[default]
    Other,
}

/// Event fact
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Event {
    pub event: EventType,
    pub date: Option<String>,
    pub place: Option<String>,
    pub citations: Vec<SourceCitation>,
}

impl TryFrom<&str> for Event {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let etype = match value {
            "ADOP" => EventType::Adoption,
            "BIRT" => EventType::Birth,
            "BURI" => EventType::Burial,
            "CHR" => EventType::Christening,
            "DEAT" => EventType::Death,
            "MARR" => EventType::Marriage,
            "RESI" => EventType::Residence,
            "OTHER" => EventType::Other,
            _ => return Err(format!("Unrecognized event tag: {}", value)),
        };
        let mut event = Event::default();
        event.event = etype;
        Ok(event)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Family {
    pub xref: Option<Xref>,
    pub individual1: Option<Xref>, // mapped from HUSB
    pub individual2: Option<Xref>, // mapped from WIFE
    pub children: Vec<Xref>,
    pub num_children: Option<u8>,
    pub(crate) events: Vec<Event>,
}

impl Family {
    pub fn set_individual1(&mut self, xref: Xref) {
        match self.individual1 {
            Some(_) => panic!("First individual of family already exists."),
            None => self.individual1 = Some(xref),
        };
    }

    pub fn set_individual2(&mut self, xref: Xref) {
        match self.individual2 {
            Some(_) => panic!("Second individual of family already exists."),
            None => self.individual2 = Some(xref),
        };
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// Header containing GEDCOM metadata
pub struct Header {
    pub encoding: Option<String>,
    pub copyright: Option<String>,
    pub corporation: Option<String>,
    pub date: Option<String>,
    pub destinations: Vec<String>,
    pub gedcom_version: Option<String>,
    pub language: Option<String>,
    pub filename: Option<String>,
    pub note: Option<String>,
    pub sources: Vec<Source>,
    pub submitter_tag: Option<String>,
    pub submission_tag: Option<String>,
}

/// A Person within the family tree
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Individual {
    pub xref: Option<Xref>,
    pub name: Option<Name>,
    pub sex: Gender,
    pub families: Vec<FamilyLink>,
    pub custom_data: Vec<CustomData>,
    pub last_updated: Option<String>,
    pub(crate) events: Vec<Event>,
}

impl Individual {
    pub fn add_family(&mut self, link: FamilyLink) {
        let mut do_add = true;
        let xref = &link.xref;
        for FamilyLink {
            link: _,
            xref: family_xref,
            pedigree: _,
        } in &self.families
        {
            if family_xref.as_str() == xref.as_str() {
                do_add = false;
            }
        }
        if do_add {
            self.families.push(link);
        }
    }
}

/// Gender of an `Individual`
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Gender {
    #[default]
    Male,
    Female,
    // come at me LDS, i support "N" as a gender value
    Nonbinary,
    Unknown,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) enum FamilyLinkType {
    #[default]
    Spouse,
    Child,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Pedigree {
    Adopted,
    Birth,
    Foster,
    Sealing,
}

impl TryFrom<&str> for Pedigree {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "adopted" => Ok(Pedigree::Adopted),
            "birth" => Ok(Pedigree::Birth),
            "foster" => Ok(Pedigree::Foster),
            "sealing" => Ok(Pedigree::Sealing),
            _ => Err(format!("Unrecognized family link pedigree: {}", value)),
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub value: Option<String>,
    pub given: Option<String>,
    pub surname: Option<String>,
    pub prefix: Option<String>,
    pub surname_prefix: Option<String>,
    pub suffix: Option<String>,
}

// TODO
/// Multimedia item
#[derive(Debug, Serialize, Deserialize)]
pub struct Media {}

/// Data repository, the `REPO` tag
#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    /// Optional reference to link to this repo
    pub xref: Option<Xref>,
    /// Name of the repository
    pub name: Option<String>,
    /// Physical address of the data repository
    pub address: Option<Address>,
}

/// Citation linking a genealogy fact to a data `Source`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceCitation {
    /// Reference to the `Source`
    pub xref: Xref,
    /// Page number of source
    pub page: Option<String>,
}

/// Citation linking a `Source` to a data `Repository`
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RepoCitation {
    /// Reference to the `Repository`
    pub xref: Xref,
    /// Call number to find the source at this repository
    pub call_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomData {
    pub tag: String,
    pub value: String,
}

/// Submitter of the data, ie. who reported the genealogy fact
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Submitter {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    /// Name of the submitter
    pub name: Option<String>,
    /// Physical address of the submitter
    pub address: Option<Address>,
    /// Phone number of the submitter
    pub phone: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SourceData {
    pub(crate) events: Vec<Event>,
    pub agency: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
/// Source for genealogy facts
pub struct Source {
    pub xref: Option<String>,
    pub data: SourceData,
    pub abbreviation: Option<String>,
    pub title: Option<String>,
    pub repo_citations: Vec<RepoCitation>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FamilyLink {
    pub xref: Xref,
    pub link: FamilyLinkType,
    pub pedigree: Option<Pedigree>,
}
