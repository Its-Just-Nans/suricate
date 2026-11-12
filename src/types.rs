//! Internal types of suricate

use std::{collections::HashMap, fmt::Display};

use oxidged::{Dataset, model::XrefId};

/// Data extracted from the file
#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone)]
pub(crate) struct TreeData {
    /// List of individuals
    #[serde(skip)]
    pub(crate) individuals: HashMap<Id, Individual>,
    /// List of families
    #[serde(skip)]
    pub(crate) families: HashMap<Id, Family>,
}

/// Internal id
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Id(u32);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    /// Show the id pretty
    pub(crate) fn to_string_pretty(&self) -> String {
        format!("#{self}")
    }
}

/// Indiviudal
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub(crate) struct Individual {
    /// id
    pub(crate) id: Id,
    /// name
    pub(crate) name: String,
    /// sex
    pub(crate) sex: String,
    /// spouse family
    pub(crate) spouse_of: Vec<Id>,
    /// child family
    pub(crate) child_of: Vec<Id>,
}

/// Family
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub(crate) struct Family {
    /// id
    pub(crate) id: Id,
    /// indi 1
    pub(crate) indi1: Option<Id>,
    /// indi 2
    pub(crate) indi2: Option<Id>,
    /// children
    pub(crate) children: Vec<Id>,
}

/// Parse the xref id
fn parse_xref_id(s: &str) -> u32 {
    s.get(2..s.len().saturating_sub(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Fully parse the xref id
fn full_parse_xref_id(data: &Dataset, xref: Option<XrefId>) -> u32 {
    xref.map_or(0, |x| parse_xref_id(data.store().xref(x)))
}

impl TreeData {
    /// Create a new data
    pub(crate) fn new(text: impl Into<String>) -> Self {
        let dataset = Dataset::parse(text);
        let individuals = dataset
            .individuals()
            .into_iter()
            .map(|f| {
                let key = Id(full_parse_xref_id(&dataset, f.xref()));
                let indi = Individual {
                    id: key.clone(),
                    sex: f.sex().map_or("N".to_string(), |s| format!("{s:?}")),
                    name: f
                        .names()
                        .first()
                        .map_or("No name".to_string(), oxidged::model::View::full),
                    spouse_of: f
                        .spouse_of()
                        .into_iter()
                        .map(|f| Id(full_parse_xref_id(&dataset, f.family())))
                        .collect(),
                    child_of: f
                        .child_of()
                        .into_iter()
                        .map(|f| Id(full_parse_xref_id(&dataset, f.family())))
                        .collect(),
                };
                (key, indi)
            })
            .collect();
        let families = dataset
            .families()
            .into_iter()
            .map(|f| {
                let key = Id(full_parse_xref_id(&dataset, f.xref()));
                let fam = Family {
                    id: key.clone(),
                    indi1: Some(Id(full_parse_xref_id(&dataset, f.husband_id()))),
                    indi2: Some(Id(full_parse_xref_id(&dataset, f.wife_id()))),
                    children: f
                        .children()
                        .into_iter()
                        .map(|f| Id(full_parse_xref_id(&dataset, f.individual())))
                        .collect(),
                };
                (key, fam)
            })
            .collect();
        Self {
            individuals,
            families,
        }
    }
}
