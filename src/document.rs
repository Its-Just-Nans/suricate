//! suricate document

use bladvak::AppError;
use bladvak::eframe::egui;
use bladvak::utils::document::DocumentTrait;
use encoding_rs::mem::decode_latin1;
use ged_io::Gedcom;
use ged_io::types::family::Family;
use ged_io::types::individual::Individual;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::central_panel::build_family_nodes;
use crate::windows::WindowsData;

/// Data associated to a node
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct NodeData {
    /// Xref of the node
    pub(crate) xref: String,
    /// Name of the individual
    pub(crate) name: String,
}

/// Node to render
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct Node {
    /// Node id
    pub id: egui::Id,
    /// Node position
    pub pos: egui::Pos2, // center position in scene space
    /// Node size
    pub size: egui::Vec2,
    /// Xref
    pub data: NodeData,
    /// Is node selected
    pub selected: bool,
}

/// Data extracted from the file
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct TreeData {
    /// List of individuals
    #[serde(skip)]
    pub(crate) individuals: HashMap<String, Individual>,
    /// List of families
    #[serde(skip)]
    pub(crate) families: HashMap<String, Family>,
}

/// A suricate document
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Document {
    /// Current scene zoom
    pub(crate) scene_rect: egui::Rect,
    /// Filename
    pub(crate) filename: PathBuf,
    /// Data
    pub(crate) data: TreeData,
    /// Node to render
    pub(crate) nodes: Vec<Node>,
    /// Selected Node
    pub(crate) selected: Option<String>,
    /// Windows Data
    pub(crate) windows_data: WindowsData,
}

impl Default for Document {
    fn default() -> Self {
        let (filename, bytes) = Self::load_default_file();
        let doc = Self::try_new(filename, bytes);
        doc.unwrap_or_else(|_| Self {
            scene_rect: egui::Rect::NAN,
            filename: PathBuf::new(),
            data: TreeData::default(),
            nodes: vec![],
            selected: None,
            windows_data: WindowsData::new(),
        })
    }
}

impl DocumentTrait for Document {
    fn path(&self) -> &Path {
        &self.filename
    }
}

/// default file
const ASSET: &[u8] = include_bytes!("../assets/royal92.ged");

impl Document {
    /// Try to create new
    pub(crate) fn try_new(path: PathBuf, bytes: &[u8]) -> Result<Self, AppError> {
        let gedcom_source = match std::str::from_utf8(bytes) {
            Ok(s) => Cow::Borrowed(s),
            Err(_e) => decode_latin1(bytes),
        };

        let mut gedcom =
            Gedcom::new(gedcom_source.chars()).map_err(|e| format!("gedcom error: {e}"))?;
        let gedcom_data = gedcom
            .parse_data()
            .map_err(|e| format!("gedcom error: {e}"))?;

        let tree_data = TreeData {
            individuals: gedcom_data
                .individuals
                .into_iter()
                .map(|f| {
                    let key = f.xref.clone().unwrap_or("1".to_string());
                    (key, f)
                })
                .collect(),
            families: gedcom_data
                .families
                .into_iter()
                .map(|f| {
                    let key = f.xref.clone().unwrap_or("1".to_string());
                    (key, f)
                })
                .collect(),
        };
        let nodes = build_family_nodes(&tree_data.individuals, &tree_data.families);

        Ok(Self {
            data: tree_data,
            filename: path,
            nodes,
            scene_rect: egui::Rect::NAN,
            selected: None,
            windows_data: WindowsData::new(),
        })
    }
    /// Load the default image
    pub(crate) fn load_default_file() -> (PathBuf, &'static [u8]) {
        let filename = PathBuf::from("royal92.ged");
        (filename, ASSET)
    }

    /// Mark data as stale
    #[allow(unused)]
    pub(crate) fn stale(&mut self) {
        self.windows_data.reset();
    }
}
