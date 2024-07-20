use crate::dnd::item::Item;

/// Two unified lists that are separated by a horizontal line.
pub struct TwoLists {
    list_1: Vec<Item>,
    list_2: Vec<Item>,
}

fn draw_list(
    list: &[Item],
    ui: &mut egui::Ui,
    dnd_item_iter: &mut hello_egui::dnd::item_iterator::ItemIterator,
    dnd_idx: &mut usize,
) {
    for item in list {
        dnd_item_iter.next(
            ui,
            egui::Id::new(item.id),
            *dnd_idx,
            true,
            |ui, dnd_item| {
                dnd_item.ui(ui, |ui, handle, _item_state| {
                    item.ui(ui, handle);
                })
            },
        );

        *dnd_idx += 1;
    }
}

impl TwoLists {
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
        let mut dnd_idx = 0usize;
        let response =
            hello_egui::dnd::dnd(ui, "dnd_two_lists").show_custom(|ui, dnd_item_iter| {
                // Draw first list
                draw_list(&self.list_1, ui, dnd_item_iter, &mut dnd_idx);

                // Draw separator
                dnd_item_iter.next(
                    ui,
                    egui::Id::new("dnd_two_lists_separator"),
                    dnd_idx,
                    true,
                    |ui, dnd_item| {
                        dnd_item.ui(ui, |ui, _handle, _item_state| {
                            ui.separator();
                        })
                    },
                );
                dnd_idx += 1;

                // Draw second list
                draw_list(&self.list_2, ui, dnd_item_iter, &mut dnd_idx);
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
