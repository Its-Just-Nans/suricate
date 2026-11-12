//! Central panel UI

use bladvak::eframe::egui::{self, Pos2, Sense};

use crate::SuricateApp;
use crate::document::Node;
use crate::document::NodeData;
use crate::types::Id;
use crate::types::{Family, Individual};

impl SuricateApp {
    /// Central panel
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        if self.documents.get_current_doc_mut().is_none() {
            bladvak::utils::central_ui(ui, |ui| {
                ui.heading(concat!("Welcome to ", env!("CARGO_PKG_NAME")));
                ui.label("No document opened");
            });
            return;
        }
        let _rect = ui.available_rect_before_wrap();
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        egui::Scene::new()
            .max_inner_size([350.0, 1000.0])
            .zoom_range(0.1..=50.0)
            .show(ui, &mut document.scene_rect, |ui| {
                let bg_r: egui::Response = ui.response();
                if bg_r.rect.is_finite() {
                    self.grid.draw(&bg_r.rect, ui.painter());
                }

                for node in &mut document.nodes {
                    let node_rect = egui::Rect::from_center_size(node.pos, node.size);

                    let node_ui = &mut ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(node_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min))
                            .id_salt(node.id)
                            .sense(egui::Sense::click_and_drag()),
                    );

                    egui::Frame::new()
                        .inner_margin(5)
                        .fill(if node.selected {
                            egui::Color32::from_rgb(70, 130, 200)
                        } else {
                            egui::Color32::from_rgb(45, 45, 55)
                        })
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(100, 100, 120),
                        ))
                        .corner_radius(6.0)
                        .show(node_ui, |ui| {
                            let name =
                                if let Some(indi) = document.data.individuals.get(&node.data.id) {
                                    &indi.name
                                } else {
                                    "Individual not found"
                                };
                            // Title bar
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(name)
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                );
                            });
                            ui.separator();
                            // Node content goes here
                        });
                    let response = ui.interact(
                        node_rect,
                        ui.scope_id().with(format!("{}-int", node.id.value())),
                        Sense::click_and_drag(),
                    );
                    // Drag to move
                    // if response.dragged() {
                    //     node.pos += response.drag_delta();
                    // }

                    // Click to select (deselect others outside this loop if needed)
                    if response.clicked() {
                        node.selected = !node.selected;
                        if node.selected {
                            document.selected = Some(node.data.id.clone());
                        } else {
                            document.selected = None;
                        }
                    } else if let Some(nn) = &document.selected {
                        if *nn != node.data.id {
                            node.selected = false;
                        }
                    } else {
                        node.selected = false;
                    }
                }
            });
    }
}

use std::collections::HashMap;
use std::collections::HashSet;

/// Node width
const NODE_W: f32 = 180.0;
/// Node height
const NODE_H: f32 = 60.0;
/// horizontal gap between nodes
const H_GAP: f32 = 30.0;
/// vertical gap between generations
const V_GAP: f32 = 80.0;

/// Build the tree
pub fn build_family_nodes(
    individuals: &HashMap<Id, Individual>,
    families: &HashMap<Id, Family>,
) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut placed: HashMap<Id, Pos2> = HashMap::new();

    // 1. Find root individuals: those who appear only as Spouse, never as Child
    let children_xrefs: HashSet<Id> = individuals
        .values()
        .flat_map(|ind| ind.child_of.iter().map(|_| ind.id.clone()))
        .collect();

    let roots: Vec<&Individual> = individuals
        .values()
        .filter(|ind| !children_xrefs.contains(&ind.id))
        .collect();

    // 2. Place generation by generation using BFS
    let mut generation: Vec<Vec<Id>> = Vec::new();
    let mut visited_fams: HashSet<Id> = HashSet::default();

    // Generation 0: root individuals grouped by their spouse family
    let root_xrefs: Vec<Id> = roots.iter().map(|i| i.id.clone()).collect();
    generation.push(root_xrefs.clone());

    // Walk down through families
    let mut current_gen = root_xrefs;
    loop {
        let mut next_gen: Vec<Id> = Vec::new();
        for xref in &current_gen {
            if let Some(ind) = individuals.get(xref) {
                for family_id in &ind.spouse_of {
                    if visited_fams.contains(family_id) {
                        continue;
                    }
                    visited_fams.insert(family_id.clone());

                    if let Some(fam) = families.get(family_id) {
                        for child_xref in &fam.children {
                            if !next_gen.contains(child_xref) {
                                next_gen.push(child_xref.clone());
                            }
                        }
                    }
                }
            }
        }
        if next_gen.is_empty() {
            break;
        }
        generation.push(next_gen.clone());
        current_gen = next_gen;
    }

    // 3. Assign positions: center each generation horizontally
    for (gen_idx, members) in generation.iter().enumerate() {
        let members_len = members.len();
        #[allow(clippy::cast_precision_loss)]
        let total_w = members_len as f32 * NODE_W + (members_len.saturating_sub(1)) as f32 * H_GAP;
        let start_x = -total_w / 2.0;
        #[allow(clippy::cast_precision_loss)]
        let y = gen_idx as f32 * (NODE_H + V_GAP);

        for (i, xref) in members.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let x = start_x + i as f32 * (NODE_W + H_GAP);
            let pos = Pos2::new(x, y);
            placed.insert(xref.clone(), pos);

            nodes.push(Node {
                id: egui::Id::unique(xref),
                pos,
                size: egui::vec2(180.0, 80.0),
                data: NodeData {
                    id: xref.clone(),
                    name: individuals
                        .get(xref)
                        .map(|i| i.name.clone())
                        .unwrap_or_default(),
                },
                selected: false,
            });
        }
    }

    // 4. Insert spouse pairs side-by-side within their generation
    // (optional: nudge ind1 and ind2 of each family closer together)
    for fam in families.values() {
        let ind1 = &fam.indi1;
        let ind2 = &fam.indi2;
        if let (Some(a), Some(b)) = (ind1, ind2) {
            let a_id = egui::Id::unique(a);
            let b_id = egui::Id::unique(b);
            if let (Some(pa), Some(pb)) = (placed.get(a), placed.get(b)) {
                // If they ended up on the same row, nudge them adjacent
                if (pa.y - pb.y).abs() < 1.0 {
                    let mid_x = f32::midpoint(pa.x, pb.x);
                    if let Some(node) = nodes.iter_mut().find(|n| n.id == a_id) {
                        node.pos.x = mid_x - f32::midpoint(NODE_W, H_GAP);
                    }
                    if let Some(node) = nodes.iter_mut().find(|n| n.id == b_id) {
                        node.pos.x = mid_x + f32::midpoint(NODE_W, H_GAP);
                    }
                }
            }
        }
    }

    nodes
}
