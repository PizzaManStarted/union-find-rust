use eframe::egui;
use egui::{Color32, Painter, Pos2, Rect, RichText, Scene, Stroke};
use egui_file_dialog::FileDialog;
use ringbuf::{
    HeapRb,
    traits::{Consumer, RingBuffer},
};
use strum::IntoEnumIterator;

use std::path::PathBuf;

use union_find_rust::union_find::{UnionFind, UnionFindChoice};

// fn main() -> io::Result<()> {
//     read_file::<WeightedQuickUnion>("resources/large.txt")
// }

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Union Finding Playground",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(MyApp::default()))
        }),
    )
}

struct MyApp {
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
    picked_file_name: Option<String>,
    picked_algo: UnionFindChoice,

    show_error: bool,

    // union find
    union_find: Option<UnionFind>,
    copy_sites: Option<Vec<usize>>,

    ring_buff: Option<HeapRb<(usize, usize, bool)>>,

    tree_view: bool,

    to_reload: bool,

    // Scene settings
    scene_rect: Rect,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            scene_rect: Rect::ZERO,
            file_dialog: Default::default(),
            picked_algo: Default::default(),
            picked_file: Default::default(),
            picked_file_name: Default::default(),
            union_find: Default::default(),
            copy_sites: Default::default(),
            ring_buff: Default::default(),
            to_reload: false,
            tree_view: false,
            show_error: false,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ctx: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Add file:");
                if ui
                    .button(if self.picked_file.is_none() {
                        "Pick file".to_string()
                    } else {
                        self.picked_file_name.as_ref().expect("msg").to_string()
                    })
                    .clicked()
                {
                    self.file_dialog.pick_file();
                }

                ui.separator();

                // Choice of algorithm
                ui.label("Algorithm:");
                let mut new_choice = self.picked_algo.clone();

                egui::ComboBox::from_id_salt("algo_combo_box")
                    .selected_text(format!("{:?}", self.picked_algo))
                    .show_ui(ui, |ui| {
                        UnionFindChoice::iter().for_each(|method| {
                            ui.selectable_value(
                                &mut new_choice,
                                method.clone(),
                                format!("{method:?}"),
                            );
                        });
                    });

                ui.separator();

                ui.add_enabled_ui(self.union_find.is_some(), |ui| {
                    if let Some(uf) = &mut self.union_find
                        && let Some(rb) = &mut self.ring_buff
                    {
                        if ui.button("Next").clicked() {
                            // Copy BEFORE moving on to the next step
                            self.copy_sites = Some(uf.get_sites().clone());
                            let res = uf.next();
                            if let Some(res) = res {
                                rb.push_overwrite(res);
                            }
                        }

                        if ui.button("Finish").clicked() {
                            for res in uf.into_iter() {
                                rb.push_overwrite(res);
                            }
                            // Copy AFTER finishing everything (we do not care about the in btw steps)
                            self.copy_sites = Some(uf.get_sites().clone());
                        }

                        if ui.button("Reset").clicked() {
                            self.to_reload = true;
                        }

                        ui.separator();

                        ui.add_enabled_ui(self.picked_algo != UnionFindChoice::QuickFind, |ui| {
                            ui.checkbox(&mut self.tree_view, "Show tree view")
                                .on_disabled_hover_text(
                                    "QuickFind does not have really work as a tree representation",
                                );
                        });
                    }
                });

                if self.picked_algo != new_choice {
                    self.to_reload = true;
                    self.picked_algo = new_choice;
                }
            });

            self.file_dialog.update(ui);
            // Check if the user picked a file.
            if let Some(path) = self.file_dialog.take_picked() {
                self.to_reload = true;

                self.picked_file_name = Some(
                    path.file_name()
                        .expect("has a file name")
                        .to_str()
                        .expect("correct string path")
                        .to_string(),
                );
                self.picked_file = Some(path.to_path_buf());
                // Create union find iterator
            }

            // reload iterator
            if self.to_reload
                && let Some(path) = &self.picked_file
            {
                let uf = UnionFind::new(path, &self.picked_algo).expect("no io error");
                if uf.get_n() > 1000 {
                    self.show_error = true;
                    self.picked_file_name = None;
                    self.picked_file = None;
                } else {
                    self.ring_buff = Some(HeapRb::new(uf.get_n()));
                    self.copy_sites = Some(uf.get_sites().clone());
                    self.union_find = Some(uf);
                }

                self.to_reload = false;
            }

            ui.separator();
            if self.picked_file.is_some() {
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::Max).with_cross_justify(true),
                    |ui| {
                        ui.vertical(|ui| {
                            egui::ScrollArea::vertical()
                                .max_width(150.)
                                .auto_shrink([false; 2])
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                                .show(ui, |ui| {
                                    if let Some(rb) = &self.ring_buff
                                        && let Some(uf) = &self.union_find
                                    {
                                        for (p, q, b) in rb.iter() {
                                            ui.label(
                                                RichText::new(format!("{p} - {q}"))
                                                    .color(if *b {
                                                        Color32::BLACK
                                                    } else {
                                                        Color32::LIGHT_RED
                                                    })
                                                    .size(25.),
                                            );
                                        }
                                        ui.label(
                                            RichText::new(
                                                // Show next line if it exists
                                                match uf.peak_next() {
                                                    Some((p, q)) => {
                                                        format!("Next: {p} - {q}")
                                                    }
                                                    None => "Finished".to_string(),
                                                },
                                            )
                                            .color(Color32::GRAY)
                                            .size(25.),
                                        );
                                    }
                                });
                        });
                        ui.separator();

                        let scene = Scene::new()
                            // .max_inner_size([350.0, 1000.0])
                            .zoom_range(0.1..=2.0);
                        let mut inner_rect = Rect::NAN;

                        let response = scene
                            .show(ui, &mut self.scene_rect, |ui| {
                                let Some(uf) = &self.union_find else { return };
                                let Some(sites) = &self.copy_sites else {
                                    return;
                                };

                                let painter = ui.painter();
                                if self.tree_view {
                                    tree_display(uf, sites, painter);
                                } else {
                                    grid_display(uf, sites, painter);
                                }

                                inner_rect = ui.min_rect();
                            })
                            .response;

                        if response.double_clicked() {
                            self.scene_rect = inner_rect;
                        }
                    },
                );
            }
        });
        if self.show_error {
            egui::Modal::new("alert_modal".into()).show(ctx, |ui| {
                ui.heading("File to big error");
                ui.label("The given file as more than 1000 sites.");
                if ui.button("Fermer").clicked() {
                    self.show_error = false;
                }
            });
        }
    }
}

fn grid_display(uf: &UnionFind, cp_sites: &[usize], painter: &Painter) {
    let mut center = Pos2::ZERO;
    let size = 30.;

    for (i, &val) in uf.get_sites().iter().enumerate() {
        let rect = Rect::from_min_max(
            (center.x - size, center.y - size).into(),
            (center.x + size, center.y + size).into(),
        );
        painter.rect_filled(
            rect,
            0,
            if cp_sites[i] != uf.get_sites()[i] {
                Color32::LIGHT_RED
            } else {
                Color32::WHITE
            },
        );
        painter.rect_stroke(
            rect,
            0,
            Stroke::new(2.0, Color32::BLACK),
            egui::StrokeKind::Middle,
        );

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            val.to_string(),
            egui::FontId::proportional(25.0),
            if cp_sites[i] != uf.get_sites()[i] {
                Color32::RED
            } else {
                Color32::BLACK
            },
        );

        center = (center.x + size * 2., center.y).into();
    }
}

fn tree_display(uf: &UnionFind, cp_sites: &[usize], painter: &Painter) {
    let radius = 28.;
    let (roots, children) = uf.to_tree();

    let mut positions = vec![Pos2::ZERO; uf.get_n()];
    let mut next_x = 60.0;

    for &root in &roots {
        tree_layout(root, 0, &children, &mut positions, &mut next_x);
        next_x += 80.0; // next tree
    }

    for (i, &parent) in uf.get_sites().iter().enumerate() {
        if i != parent {
            painter.line_segment(
                [positions[parent], positions[i]],
                Stroke::new(2.0, Color32::BLACK),
            );
        }
    }

    for (i, pos) in positions.iter().enumerate() {
        painter.circle_filled(
            *pos,
            radius,
            if cp_sites[i] != uf.get_sites()[i] {
                Color32::LIGHT_RED
            } else {
                Color32::WHITE
            },
        );
        painter.circle_stroke(
            *pos,
            radius,
            Stroke::new(
                2.0,
                if cp_sites[i] != uf.get_sites()[i] {
                    Color32::RED
                } else {
                    Color32::BLACK
                },
            ),
        );

        painter.text(
            *pos,
            egui::Align2::CENTER_CENTER,
            i.to_string(),
            egui::FontId::proportional(26.0),
            Color32::BLACK,
        );
    }
}

fn tree_layout(
    node: usize,
    depth: usize,
    children: &[Vec<usize>],
    positions: &mut [Pos2],
    next_x: &mut f32,
) {
    // if is Root
    if children[node].is_empty() {
        positions[node] = Pos2::new(*next_x, depth as f32 * 120.0);
        *next_x += 90.0;
    } else {
        for &child in &children[node] {
            tree_layout(child, depth + 1, children, positions, next_x);
        }

        let first = positions[children[node].first().copied().expect("at least one val")].x;
        let last = positions[children[node].last().copied().expect("at least one val")].x;

        // Place it between the position of the first and last children
        positions[node] = Pos2::new((first + last) * 0.5, depth as f32 * 120.0);
    }
}
