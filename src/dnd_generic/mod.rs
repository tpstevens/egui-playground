mod demo_vertical;
mod generic_list;

// TODO: move this and demo_vertical to binary inside new dnd_generic crate
pub fn run_demo() -> eframe::Result {
    let mut vertical_items = demo_vertical::ItemCache::demo_new(60000, 3, 2);
    let mut vertical_dnd = generic_list::DndItemList::<usize, usize>::default();

    eframe::run_simple_native(
        "egui_playground_dnd_generic",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::containers::panel::CentralPanel::default().show(ctx, |ui| {
                ui.columns(1, |columns| {
                    demo_vertical(&mut columns[0], &mut vertical_items, &mut vertical_dnd);
                });
            });
        },
    )
}

fn demo_base(ui: &mut egui::Ui, heading: &str, description: &str, f: impl FnOnce(&mut egui::Ui)) {
    // Ensure that text isn't justified
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.heading(heading);
        ui.label(description);
    });
    ui.separator();
    f(ui);
}

fn demo_vertical(
    ui: &mut egui::Ui,
    items: &mut demo_vertical::ItemCache,
    dnd: &mut generic_list::DndItemList<usize, usize>,
) {
    demo_base(
        ui,
        "Generic vertical nested list",
        "(items can be collapsed and dragged between any list)",
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("demo_vertical_scroll")
                .show(ui, |ui| {
                    let root_list_id = items.root_list_id();

                    generic_list::ui(
                        ui,
                        egui::Id::new("demo_vertical"),
                        items,
                        &root_list_id,
                        dnd,
                    );
                });
        },
    );
}
