use egui::{Color32, Context, ScrollArea, Stroke, Vec2};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use session_store::SessionStore;
use telemetry_ac::AcSample;

use crate::signals::{SessionSignals, Signal};

// Palette de couleurs pour les courbes
const COLORS: &[Color32] = &[
    Color32::from_rgb(235, 100,  60),  // orange-rouge
    Color32::from_rgb( 80, 180, 230),  // bleu clair
    Color32::from_rgb(100, 210, 120),  // vert
    Color32::from_rgb(220, 190,  60),  // jaune
    Color32::from_rgb(180, 100, 220),  // violet
    Color32::from_rgb( 60, 200, 180),  // cyan
    Color32::from_rgb(230, 130, 180),  // rose
    Color32::from_rgb(160, 160, 160),  // gris
];

pub struct ViewerApp {
    file_path:  String,
    signals:    Option<SessionSignals>,
    error:      Option<String>,
    /// Indices des signaux sélectionnés pour l'affichage (depuis la liste gauche)
    selected:   Vec<bool>,
    /// Nb de panneaux en colonne (1, 2 ou "tout superposé")
    layout:     Layout,
}

#[derive(PartialEq, Clone, Copy)]
enum Layout {
    Overlay,   // toutes les courbes sélectionnées sur un seul plot
    Stacked,   // un plot par courbe, alignés verticalement, axe X synchronisé
}

impl ViewerApp {
    pub fn load(_cc: &eframe::CreationContext, path: &str) -> Self {
        let mut app = Self {
            file_path: path.to_string(),
            signals:   None,
            error:     None,
            selected:  Vec::new(),
            layout:    Layout::Stacked,
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        match SessionStore::load::<AcSample, _>(&self.file_path) {
            Ok((meta, samples)) => {
                let ss = SessionSignals::from_ac_samples(&samples);
                let n = ss.signals.len();
                // Sélection par défaut : vitesse, gaz, frein, braquage
                let mut sel = vec![false; n];
                for i in 0..4.min(n) { sel[i] = true; }
                self.selected = sel;
                self.signals  = Some(ss);
                self.error    = None;
                println!("Session chargée : {} samples, {:.1}s — {}",
                    samples.len(), meta.duration_s, meta.simulator);
            }
            Err(e) => {
                self.error = Some(format!("Erreur : {e}"));
            }
        }
    }

    /// Renvoie les signaux actuellement sélectionnés avec leur couleur
    fn active_signals<'a>(&'a self, ss: &'a SessionSignals) -> Vec<(&'a Signal, Color32)> {
        ss.signals.iter()
            .enumerate()
            .filter(|(i, _)| self.selected.get(*i).copied().unwrap_or(false))
            .enumerate()
            .map(|(color_idx, (_, sig))| (sig, COLORS[color_idx % COLORS.len()]))
            .collect()
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // ── Panneau gauche : liste des signaux ───────────────────────────
        egui::SidePanel::left("signal_list")
            .resizable(true)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Signaux");
                ui.separator();

                // Boutons layout
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.layout, Layout::Stacked,  "Empilés");
                    ui.selectable_value(&mut self.layout, Layout::Overlay,  "Superposés");
                });
                ui.separator();

                // Boutons tout sélectionner / désélectionner
                ui.horizontal(|ui| {
                    if ui.small_button("Tout").clicked() {
                        self.selected.iter_mut().for_each(|s| *s = true);
                    }
                    if ui.small_button("Rien").clicked() {
                        self.selected.iter_mut().for_each(|s| *s = false);
                    }
                });
                ui.add_space(4.0);

                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(ss) = &self.signals {
                        let mut color_idx = 0;
                        for (i, sig) in ss.signals.iter().enumerate() {
                            let selected = self.selected.get_mut(i).unwrap();
                            let mut color = Color32::GRAY;
                            if *selected {
                                color = COLORS[color_idx % COLORS.len()];
                                color_idx = color_idx + 1;
                            }
                            // let color = if *selected {
                            //     // Compte combien de signaux avant celui-ci sont sélectionnés
                            //     // let color_idx = self.selected[..i].iter().filter(|&&s| s).count();
                            //     color_idx = color_idx + 1;
                            //     COLORS[color_idx % COLORS.len()]
                            // } else {
                            //     Color32::GRAY
                            // };

                            ui.horizontal(|ui| {
                                // Carré de couleur
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(12.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, color);

                                // Checkbox + label
                                let label = format!("{} ({})", sig.name, sig.unit);
                                ui.checkbox(selected, label);
                            });

                            // Min/max en petit sous le nom
                            if *selected {
                                ui.label(
                                    egui::RichText::new(
                                        format!("  [{:.3} … {:.3}]", sig.min(), sig.max())
                                    ).small().color(Color32::DARK_GRAY)
                                );
                            }
                        }
                    }
                });

                if let Some(err) = &self.error {
                    ui.colored_label(Color32::RED, err);
                }
            });

        // ── Zone centrale : plots ─────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(ss) = &self.signals else {
                ui.centered_and_justified(|ui| {
                    ui.label(self.error.as_deref().unwrap_or("Chargement…"));
                });
                return;
            };

            let active = self.active_signals(ss);

            if active.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Sélectionnez des signaux dans la liste à gauche.");
                });
                return;
            }

            match self.layout {
                Layout::Overlay => {
                    // Tous les signaux sur un seul plot
                    let plot = Plot::new("overlay")
                        .legend(Legend::default())
                        .x_axis_label("Temps (s)")
                        .height(ui.available_height());

                    plot.show(ui, |plot_ui| {
                        for (sig, color) in &active {
                            let points: PlotPoints = sig.times.iter()
                                .zip(sig.values.iter())
                                .map(|(&t, &v)| [t, v])
                                .collect();
                            plot_ui.line(
                                Line::new(points)
                                    .name(format!("{} ({})", sig.name, sig.unit))
                                    .stroke(Stroke::new(1.5, *color))
                            );
                        }
                    });
                }

                Layout::Stacked => {
                    // Un plot par signal, hauteur partagée équitablement
                    let n = active.len();
                    let available_h = ui.available_height();
                    let plot_h = (available_h / n as f32).max(60.0);

                    // Axe X partagé : on utilise le même `link_axis` id
                    let link_id = ui.id().with("x_link");

                    ScrollArea::vertical().show(ui, |ui| {
                        for (sig, color) in &active {
                            let plot = Plot::new(sig.name)
                                .height(plot_h)
                                .legend(Legend::default())
                                .link_axis(link_id, true) // synchronise l'axe X
                                .y_axis_label(format!("{} ({})", sig.name, sig.unit))
                                .show_x(true)
                                .show_y(true);

                            plot.show(ui, |plot_ui| {
                                let points: PlotPoints = sig.times.iter()
                                    .zip(sig.values.iter())
                                    .map(|(&t, &v)| [t, v])
                                    .collect();
                                plot_ui.line(
                                    Line::new(points)
                                        .name(sig.name)
                                        .stroke(Stroke::new(1.5, *color))
                                );
                            });

                            ui.add_space(2.0);
                        }
                    });
                }
            }
        });
    }
}