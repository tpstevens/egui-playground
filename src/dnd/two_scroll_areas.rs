use crate::dnd::item::Item;

pub struct TwoScrollAreas {
    list_1: Vec<Item>,
    list_2: Vec<Item>,
}

impl TwoScrollAreas {
    pub fn new(start_id: usize, num_items_per_area: usize) -> Self {
        let mut list_1 = Vec::<Item>::new();
        let mut list_2 = Vec::<Item>::new();

        for i in 0..num_items_per_area {
            list_1.push(Item::new(start_id + i));
            list_2.push(Item::new(start_id + num_items_per_area + i));
        }

        Self { list_1, list_2 }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut idx = 0usize;
        let response = hello_egui::dnd::dnd(ui, "dnd_two_scroll_areas").show_custom(|ui, iter| {
            let (first, _) = ui.max_rect().split_top_bottom_at_fraction(0.5);

            // Draw first list
            ui.allocate_ui_at_rect(first, |ui| {
                egui::ScrollArea::vertical()
                    .id_source("dnd_two_scroll_areas_top")
                    .show(ui, |ui| {
                        for item in &self.list_1 {
                            iter.next(ui, egui::Id::new(item.id), idx, true, |ui, dnd_item| {
                                dnd_item.ui(ui, |ui, handle, _item_state| {
                                    item.ui(ui, handle);
                                })
                            });

                            idx += 1;
                        }
                    });
            });

            // Draw separator
            iter.next(
                ui,
                egui::Id::new("dnd_two_scroll_areas_separator"),
                idx,
                true,
                |ui, dnd_item| {
                    dnd_item.ui(ui, |ui, _handle, _item_state| {
                        ui.separator();
                    })
                },
            );

            idx += 1;

            // Draw second list
            egui::ScrollArea::vertical()
                .id_source("dnd_two_scroll_areas_bottom")
                .show(ui, |ui| {
                    for item in &self.list_2 {
                        iter.next(ui, egui::Id::new(item.id), idx, true, |ui, dnd_item| {
                            dnd_item.ui(ui, |ui, handle, _item_state| {
                                item.ui(ui, handle);
                            })
                        });

                        idx += 1;
                    }
                });
        });

        if let Some(update) = response.update {
            crate::dnd::util::move_elements_2(
                &mut self.list_1,
                &mut self.list_2,
                update.from,
                update.to,
                1,
            );
        }
    }
}