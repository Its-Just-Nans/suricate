//! Suricate App
use bladvak::egui_extras;
use bladvak::{
    File,
    app::BladvakApp,
    errors::{AppError, ErrorManager},
    utils::grid::Grid,
};
use bladvak::{
    eframe::{
        CreationContext,
        egui::{self, pos2},
    },
    utils::is_native,
};
use ged_io::types::individual::Individual;
use ged_io::types::family::Family;
use std::collections::HashMap;
use std::{fmt::Debug, io::Cursor, path::PathBuf};

use crate::panels::FileInfo;
use crate::central_panel::build_family_nodes;

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct Node {
    pub id: egui::Id,
    pub pos: egui::Pos2, // center position in scene space
    pub size: egui::Vec2,
    pub title: String,
    pub selected: bool,
}

impl Node {
    pub fn new(id: impl std::hash::Hash, pos: egui::Pos2, title: impl Into<String>) -> Self {
        Self {
            id: egui::Id::new(id),
            pos,
            size: egui::vec2(180.0, 80.0),
            title: title.into(),
            selected: false,
        }
    }
}

/// Data extracted from the file
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct TreeData {
    /// List of individuals
    pub(crate) individuals: HashMap<String, Individual>,
    /// List of families
    pub(crate) families: HashMap<String, Family>,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct SuricateApp {
    /// Current scene zoom
    pub(crate) scene_rect: egui::Rect,
    /// Filename
    pub(crate) filename: PathBuf,
    /// Grid options
    pub(crate) grid: Grid,
    /// Data
    pub(crate) data: TreeData,

    pub(crate) nodes: Vec<Node>,

    pub(crate) selected: Option<String>,
}

impl Default for SuricateApp {
    fn default() -> Self {
        let nodes = vec![
            Node::new("image_source", pos2(30., 60.), "Image source"),
            Node::new("color_correct", pos2(240., 40.), "Color correct"),
            Node::new("blur", pos2(240., 170.), "Blur"),
            Node::new("mix", pos2(450., 100.), "Mix"),
            Node::new("output", pos2(640., 100.), "Output"),
        ];
        Self {
            scene_rect: egui::Rect::NAN,
            filename: PathBuf::new(),
            grid: Grid::default(),
            data: TreeData::default(),
            nodes,
            selected: None,
        }
    }
}

/// default image
const ASSET: &[u8] = include_bytes!("../assets/royal92.ged");

impl SuricateApp {
    /// Load the default image
    pub(crate) fn load_default_file() -> (PathBuf, Cursor<&'static [u8]>) {
        let cursor = Cursor::new(ASSET);
        let filename = PathBuf::from("royal92.ged");
        (filename, cursor)
    }
}

impl BladvakApp<'_> for SuricateApp {
    fn side_panel(
        &mut self,
        ui: &mut egui::Ui,
        func_ui: impl FnOnce(&mut egui::Ui, &mut SuricateApp),
    ) {
        ui.label("sdfg");
        ui.label("Selected");
        ui.label(format!("{:?}", self.selected));
        egui::Frame::central_panel(&ui.ctx().global_style())
            .show(ui, |panel_ui| func_ui(panel_ui, self));
    }

    fn panel_list(&self) -> Vec<Box<dyn bladvak::app::BladvakPanel<App = Self>>> {
        vec![Box::new(FileInfo)]
    }

    fn is_side_panel(&self) -> bool {
        true
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        use ged_io::Gedcom;

        // the parser takes the gedcom file contents as a chars iterator
        let gedcom_source = String::from_utf8(file.data)?;
        let mut gedcom =
            Gedcom::new(gedcom_source.chars()).map_err(|e| format!("gedcom error: {e}"))?;
        let gedcom_data = gedcom
            .parse_data()
            .map_err(|e| format!("gedcom error: {e}"))?;

        // Display file statistics
        gedcom_data.stats();
        self.filename = file.path;

        // output some stats on the gedcom contents
        self.data.individuals = gedcom_data
            .individuals
            .into_iter()
            .map(|f| {
                let key = f.xref.clone().unwrap_or("1".to_string());
                (key, f)
            })
            .collect();

        self.data.families = gedcom_data
            .families
            .into_iter()
            .map(|f| {
                let key = f.xref.clone().unwrap_or("1".to_string());
                (key, f)
            })
            .collect();

        self.nodes = build_family_nodes(&self.data.individuals, &self.data.families);
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.label(format!("Filename: {}", self.filename.display()));
    }

    fn menu_file(&mut self, _ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        //self.app_menu_file(ui, error_manager);
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
    }

    fn name() -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn repo_url() -> String {
        "https://github.com/Its-Just-Nans/suricate".to_string()
    }

    fn icon() -> &'static [u8] {
        &include_bytes!("../assets/icon-256.png")[..]
    }

    fn try_new_with_args(
        mut saved_state: Self,
        cc: &CreationContext<'_>,
        args: &[String],
    ) -> Result<Self, AppError> {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if is_native() && args.len() > 1 {
            use std::io::Read;
            let path = &args[1];
            let bytes = std::fs::read(path)?;
            let mut cursor: Cursor<&[u8]> = Cursor::new(bytes.as_ref());
            let mut buf = Vec::new();
            cursor.read_to_end(&mut buf)?;
            saved_state.handle_file(File {
                path: path.into(),
                data: buf,
            })?;
            Ok(saved_state)
        } else {
            Ok(saved_state)
        }
    }
}
