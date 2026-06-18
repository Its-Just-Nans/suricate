//! Suricate App
use bladvak::egui_extras;
use bladvak::{
    File,
    app::BladvakApp,
    errors::{AppError, ErrorManager},
    utils::grid::Grid,
};
use bladvak::{
    eframe::{CreationContext, egui},
    utils::is_native,
};
use ged_io::types::family::Family;
use ged_io::types::individual::Individual;
use std::collections::HashMap;
use std::{fmt::Debug, io::Cursor, path::PathBuf};

use crate::central_panel::build_family_nodes;
use crate::panels::FileInfo;
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
    /// Node to render
    pub(crate) nodes: Vec<Node>,
    /// Selected Node
    pub(crate) selected: Option<String>,
    /// Windows Data
    pub(crate) windows_data: WindowsData,
}

impl Default for SuricateApp {
    fn default() -> Self {
        let nodes = vec![];
        Self {
            scene_rect: egui::Rect::NAN,
            filename: PathBuf::new(),
            grid: Grid::default(),
            data: TreeData::default(),
            nodes,
            selected: None,
            windows_data: WindowsData::new(),
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

    /// Mark data as stale
    pub(crate) fn stale(&mut self) {
        self.windows_data.reset();
    }
}

impl BladvakApp<'_> for SuricateApp {
    fn side_panel(
        &mut self,
        ui: &mut egui::Ui,
        func_ui: impl FnOnce(&mut egui::Ui, &mut SuricateApp),
    ) {
        egui::Frame::central_panel(&ui.ctx().global_style()).show(ui, |panel_ui| {
            egui::ScrollArea::both().show(panel_ui, |ui| {
                self.ui_side_panel(ui);
            });
            func_ui(panel_ui, self);
        });
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
        use encoding_rs::mem::decode_latin1;
        use ged_io::Gedcom;
        use std::borrow::Cow;

        // the parser takes the gedcom file contents as a chars iterator
        let gedcom_source = match std::str::from_utf8(&file.data) {
            Ok(s) => Cow::Borrowed(s),
            Err(_e) => decode_latin1(&file.data),
        };
        let mut gedcom =
            Gedcom::new(gedcom_source.chars()).map_err(|e| format!("gedcom error: {e}"))?;
        let gedcom_data = gedcom
            .parse_data()
            .map_err(|e| format!("gedcom error: {e}"))?;

        // Display file statistics
        gedcom_data.stats();
        self.filename = file.path;
        self.stale();
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
        ui.menu_button("Windows", |ui| {
            self.windows_data.ui_top_bar(ui);
        });
        ui.separator();
        ui.label(format!("Filename: {}", self.filename.display()));
    }

    fn menu_file(&mut self, _ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        //self.app_menu_file(ui, error_manager);
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
        self.ui_windows(ui, error_manager);
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

impl SuricateApp {
    /// Side panel showing current selection
    fn ui_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("Selected");
        if let Some(xref) = &self.selected
            && let Some(indi) = self.data.individuals.get(xref)
        {
            ui.label(format!("{indi}")).on_hover_ui(|ui| {
                ui.label(format!("{indi:#?}"));
            });
            for one_family in &indi.families {
                ui.separator();
                let resp = if let Some(family) = self.data.families.get(&one_family.xref) {
                    ui.label(format!("{family}"))
                } else {
                    ui.label("Family not found")
                };
                resp.on_hover_ui(|ui| {
                    ui.label(format!("{one_family:#?}"));
                });
            }
        }
    }
}
