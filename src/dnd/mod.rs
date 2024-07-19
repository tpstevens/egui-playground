mod item;
mod multiple_lists;
mod nested_lists;
mod two_lists;
mod two_scroll_areas;
mod util;

use multiple_lists::MultipleLists;
use nested_lists::NestedLists;
use two_lists::TwoLists;
use two_scroll_areas::TwoScrollAreas;

pub fn run_demo() -> eframe::Result {
    let mut two_lists = TwoLists::new(0, 40);
    let mut two_scroll_areas = TwoScrollAreas::new(10000, 50);
    let mut multiple_lists = MultipleLists::new(6, 20000, 10);
    let (mut nested_lists, _) = NestedLists::new("nested list root".to_string(), 30000, 10);

    eframe::run_simple_native(
        "egui_playground_dnd",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::containers::panel::CentralPanel::default().show(ctx, |ui| {
                ui.columns(4, |columns| {
                    columns[0].label("Two lists");
                    columns[0].separator();
                    egui::ScrollArea::vertical()
                        .id_source("scroll_simple_separator")
                        .show(&mut columns[0], |ui| {
                            two_lists.ui(ui);
                        });

                    columns[1].label("Two scroll areas");
                    columns[1].separator();
                    two_scroll_areas.ui(&mut columns[1]);

                    columns[2].label("Multiple lists");
                    columns[2].separator();
                    egui::ScrollArea::vertical()
                        .id_source("scroll_multiple_lists")
                        .show(&mut columns[2], |ui| {
                            multiple_lists.ui(ui);
                        });

                    columns[3].label("Nested lists");
                    columns[3].separator();
                    egui::ScrollArea::vertical()
                        .id_source("scroll_nested_lists")
                        .show(&mut columns[3], |ui| {
                            nested_lists.ui(ui);
                        });
                });
            });
        },
    )
}
