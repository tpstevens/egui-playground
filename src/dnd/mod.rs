mod item;
mod multiple_lists;
mod nested_lists;
mod separate_nested_lists;
mod two_lists;
mod two_scroll_areas;
mod util;

use crate::dnd::multiple_lists::MultipleLists;
use crate::dnd::nested_lists::NestedLists;
use crate::dnd::separate_nested_lists::SeparateNestedLists;
use crate::dnd::two_lists::TwoLists;
use crate::dnd::two_scroll_areas::TwoScrollAreas;
use std::default::Default;

fn dnd_demo(ui: &mut egui::Ui, heading: &str, description: &str, f: impl FnOnce(&mut egui::Ui)) {
    // Ensure that text isn't justified
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.heading(heading);
        ui.label(description);
    });
    ui.separator();
    f(ui);
}

pub fn run_demo() -> eframe::Result {
    let mut two_lists = TwoLists::new(0, 40);
    let mut two_scroll_areas = TwoScrollAreas::new(10000, 50);
    let mut multiple_lists = MultipleLists::new(6, 20000, 10);
    let (mut separate_nested_lists, _) =
        SeparateNestedLists::new("separate nested list root".to_string(), 30000, 10);
    let (mut nested_lists, _) = NestedLists::new("nested_list_root".to_string(), 40000, 3, 2);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("egui_playground_dnd", native_options, move |ctx, _frame| {
        egui::containers::panel::CentralPanel::default().show(ctx, |ui| {
            ui.columns(5, |columns| {
                dnd_demo(
                    &mut columns[0],
                    "Two lists",
                    "(items can be dragged freely across the separator)",
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_source("scroll_simple_separator")
                            .show(ui, |ui| {
                                two_lists.ui(ui);
                            });
                    },
                );

                dnd_demo(
                    &mut columns[1],
                    "Two scroll areas",
                    "(items can be dragged between the scroll areas with some visual glitches)",
                    |ui| {
                        two_scroll_areas.ui(ui);
                    },
                );

                dnd_demo(
                    &mut columns[2],
                    "Multiple lists",
                    "(items can be dragged freely between all lists)",
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_source("scroll_multiple_lists")
                            .show(ui, |ui| {
                                multiple_lists.ui(ui);
                            });
                    },
                );

                dnd_demo(
                    &mut columns[3],
                    "Separate nested lists",
                    "(items can be dragged within a list, including items in the root list!)",
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_source("scroll_separate_nested_lists")
                            .show(ui, |ui| {
                                separate_nested_lists.ui(ui);
                            });
                    },
                );

                dnd_demo(
                    &mut columns[4],
                    "Unified nested lists",
                    "(items can be dragged between any list)",
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_source("scroll_nested_lists")
                            .show(ui, |ui| {
                                nested_lists.ui(ui);
                            });
                    },
                );
            });
        });
    })
}
