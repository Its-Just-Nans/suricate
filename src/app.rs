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
        egui::{self},
    },
    utils::is_native,
};
use std::collections::HashMap;
use std::{fmt::Debug, io::Cursor, path::PathBuf};

use crate::panels::FileInfo;

/// Data extracted from the file
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct TreeData {
    /// List of individuals
    pub(crate) individuals: HashMap<String, crate::gedcom::types::Individual>,
    /// List of families
    pub(crate) families: HashMap<u64, Vec<u64>>,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct SuricateApp {
    /// Current scene zoom
    #[serde(skip)]
    pub(crate) scene_rect: egui::Rect,
    /// Filename
    pub(crate) filename: PathBuf,
    /// Grid options
    pub(crate) grid: Grid,
    /// Data
    pub(crate) data: TreeData,
}

impl Default for SuricateApp {
    fn default() -> Self {
        Self {
            scene_rect: egui::Rect::NAN,
            filename: PathBuf::new(),
            grid: Grid::default(),
            data: TreeData::default(),
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
        egui::Frame::central_panel(&ui.ctx().global_style())
            .show(ui, |panel_ui| func_ui(panel_ui, self));
    }

    fn panel_list(&self) -> Vec<Box<dyn bladvak::app::BladvakPanel<App = Self>>> {
        vec![Box::new(FileInfo)]
    }

    fn is_side_panel(&self) -> bool {
        false
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        use crate::gedcom::parser::Parser;

        // the parser takes the gedcom file contents as a chars iterator
        let gedcom_source = String::from_utf8(file.data)?;
        let mut parser = Parser::new(gedcom_source.chars());
        let gedcom_data = parser.parse_record()?;
        self.filename = file.path;

        // output some stats on the gedcom contents
        println!("{gedcom_data:?}");
        self.data.individuals = gedcom_data
            .individuals
            .into_iter()
            .map(|f| {
                let key = f.xref.clone().unwrap_or("1".to_string());
                (key, f)
            })
            .collect();
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
