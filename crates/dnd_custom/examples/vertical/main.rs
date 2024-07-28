mod item_cache;

use dnd_custom::generic_list;

pub fn main() -> eframe::Result {
    let mut vertical_items = item_cache::ItemCache::demo_new(60000, 3, 2);
    let mut vertical_dnd = generic_list::DndItemList::<usize, usize>::default();

    eframe::run_simple_native(
        "egui_playground_dnd_custom_ex_vertical",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::containers::panel::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.heading("Generic vertical nested list");
                    ui.label("(items can be collapsed and dragged between any list)");
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_source("main_scroll")
                    .show(ui, |ui| {
                        let root_list_id = vertical_items.root_list_id();
                        generic_list::ui(
                            ui,
                            egui::Id::new("demo_vertical"),
                            &mut vertical_items,
                            &root_list_id,
                            &mut vertical_dnd,
                        );
                    });
            });
        },
    )
}
