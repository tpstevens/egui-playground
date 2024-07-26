mod collapsible_nested_lists;
mod item;
mod multiple_lists;
mod nested_lists;
mod separate_nested_lists;
mod two_lists;
mod two_scroll_areas;
mod util;

use crate::dnd::collapsible_nested_lists::CollapsibleNestedLists;
use crate::dnd::multiple_lists::MultipleLists;
use crate::dnd::nested_lists::NestedLists;
use crate::dnd::separate_nested_lists::SeparateNestedLists;
use crate::dnd::two_lists::TwoLists;
use crate::dnd::two_scroll_areas::TwoScrollAreas;
use std::default::Default;

pub fn run_demo() -> eframe::Result {
    let mut two_lists = TwoLists::new(0, 40);
    let mut two_scroll_areas = TwoScrollAreas::new(10000, 50);
    let mut multiple_lists = MultipleLists::new(6, 20000, 10);
    let (mut separate_nested_lists, _) =
        SeparateNestedLists::new("separate nested list root".to_string(), 30000, 10);
    let (mut nested_lists, _) = NestedLists::new("nested_list_root".to_string(), 40000, 3, 2);
    let (mut collapsible_nested_lists, _) =
        CollapsibleNestedLists::new("collapsible_nested_list_root".to_string(), 50000, 3, 2);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([1200.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("egui_playground_dnd", native_options, move |ctx, _frame| {
        egui::containers::panel::CentralPanel::default().show(ctx, |ui| {
            ui.columns(6, |columns| {
                demo_two_lists(&mut columns[0], &mut two_lists);
                demo_two_scroll_areas(&mut columns[1], &mut two_scroll_areas);
                demo_multiple_lists(&mut columns[2], &mut multiple_lists);
                demo_separate_nested_lists(&mut columns[3], &mut separate_nested_lists);
                demo_unified_nested_lists(&mut columns[4], &mut nested_lists);
                demo_collapsible_nested_lists(&mut columns[5], &mut collapsible_nested_lists);
            });
        });
    })
}

fn dnd_demo(ui: &mut egui::Ui, heading: &str, description: &str, f: impl FnOnce(&mut egui::Ui)) {
    // Ensure that text isn't justified
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.heading(heading);
        ui.label(description);
    });
    ui.separator();
    f(ui);
}

fn demo_two_lists(ui: &mut egui::Ui, list: &mut TwoLists) {
    dnd_demo(
        ui,
        "Two lists",
        "(items can be dragged freely across the separator)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("scroll_simple_separator")
                .show(ui, |ui| {
                    list.ui(ui);
                });
        },
    );
}

fn demo_two_scroll_areas(ui: &mut egui::Ui, list: &mut TwoScrollAreas) {
    dnd_demo(
        ui,
        "Two scroll areas",
        "(items can be dragged between the scroll areas with some visual glitches)",
        |ui| {
            list.ui(ui);
        },
    );
}

fn demo_multiple_lists(ui: &mut egui::Ui, list: &mut MultipleLists) {
    dnd_demo(
        ui,
        "Multiple lists",
        "(items can be dragged freely between all lists)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("scroll_multiple_lists")
                .show(ui, |ui| {
                    list.ui(ui);
                });
        },
    );
}

fn demo_separate_nested_lists(ui: &mut egui::Ui, list: &mut SeparateNestedLists) {
    dnd_demo(
        ui,
        "Separate nested lists",
        "(items can be dragged within a list, including items in the root list!)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("scroll_separate_nested_lists")
                .show(ui, |ui| {
                    list.ui(ui);
                });
        },
    );
}

fn demo_unified_nested_lists(ui: &mut egui::Ui, list: &mut NestedLists) {
    dnd_demo(
        ui,
        "Unified nested lists",
        "(items can be dragged between any list)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("scroll_nested_lists")
                .show(ui, |ui| {
                    list.ui(ui);
                });
        },
    );
}

fn demo_collapsible_nested_lists(ui: &mut egui::Ui, list: &mut CollapsibleNestedLists) {
    dnd_demo(
        ui,
        "Collapsible nested lists",
        "(items can be collapsed and dragged between any list)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("scroll_collapsible_nested_lists")
                .show(ui, |ui| {
                    list.ui(ui);
                });
        },
    );
}
