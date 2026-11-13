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

/// Individual
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
fn parse_xref_id(s: &str) -> Option<u32> {
    s.get(2..s.len().saturating_sub(1))
        .and_then(|v| v.parse().ok())
}

/// Fully parse the xref id
fn full_parse_xref_id(data: &Dataset, xref: Option<XrefId>) -> Option<u32> {
    xref.and_then(|x| parse_xref_id(data.store().xref(x)))
}

impl TreeData {
    /// Create a new data
    pub(crate) fn new(text: impl Into<String>) -> (Self, Vec<String>) {
        let dataset = Dataset::parse(text);
        let mut errors = vec![];
        let individuals = dataset
            .individuals()
            .into_iter()
            .filter_map(|f| {
                let Some(id) = full_parse_xref_id(&dataset, f.xref()) else {
                    errors.push(format!("Cannot parse the id of {f:?}"));
                    return None;
                };

                let spouse_of = f
                    .spouse_of()
                    .into_iter()
                    .filter_map(|f| {
                        let Some(family_id) = full_parse_xref_id(&dataset, f.family()) else {
                            errors.push(format!("Cannot parse the family_id id of {f:?}"));
                            return None;
                        };
                        Some(Id(family_id))
                    })
                    .collect();
                let child_of = f
                    .child_of()
                    .into_iter()
                    .filter_map(|f| {
                        let Some(family_id) = full_parse_xref_id(&dataset, f.family()) else {
                            errors.push(format!("Cannot parse the family_id id of {f:?}"));
                            return None;
                        };
                        Some(Id(family_id))
                    })
                    .collect();

                let key = Id(id);
                let indi = Individual {
                    id: key.clone(),
                    sex: f.sex().map_or("N".to_string(), |s| format!("{s:?}")),
                    name: f
                        .names()
                        .first()
                        .map_or("No name".to_string(), oxidged::model::View::full),
                    spouse_of,
                    child_of,
                };
                Some((key, indi))
            })
            .collect();
        let families = dataset
            .families()
            .into_iter()
            .filter_map(|f| {
                let Some(id) = full_parse_xref_id(&dataset, f.xref()) else {
                    errors.push(format!("Cannot parse the family id of {f:?}"));
                    return None;
                };

                let key = Id(id);
                let fam = Family {
                    id: key.clone(),
                    indi1: full_parse_xref_id(&dataset, f.husband_id()).map(Id),
                    indi2: full_parse_xref_id(&dataset, f.wife_id()).map(Id),
                    children: f
                        .children()
                        .into_iter()
                        .filter_map(|c| {
                            let Some(c_id) = full_parse_xref_id(&dataset, c.individual()) else {
                                errors.push(format!(
                                    "Cannot parse the indi_id id of the individual {c:?}"
                                ));
                                return None;
                            };
                            Some(Id(c_id))
                        })
                        .collect(),
                };
                Some((key, fam))
            })
            .collect();
        (
            Self {
                individuals,
                families,
            },
            errors,
        )
    }
}
