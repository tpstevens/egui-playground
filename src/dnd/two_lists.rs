use crate::dnd::item::Item;

/// Two unified lists that are separated by a horizontal line.
pub struct TwoLists {
    list_1: Vec<Item>,
    list_2: Vec<Item>,
}

type IterNextInnerFn<'a> =
    Box<dyn Fn(&mut egui::Ui, hello_egui::dnd::Handle, hello_egui::dnd::ItemState) + 'a>;

fn draw_list<'a, T>(list: &'a [Item], dnd_idx: &mut usize, mut iter_next: T)
where
    T: FnMut(egui::Id, usize, IterNextInnerFn<'a>),
{
    for item in list {
        iter_next(
            egui::Id::new(item.id),
            *dnd_idx,
            Box::new(
                |ui: &mut egui::Ui,
                 handle: hello_egui::dnd::Handle,
                 _item_state: hello_egui::dnd::ItemState| {
                    item.ui(ui, handle);
                },
            ),
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
        let mut idx = 0usize;
        let response = hello_egui::dnd::dnd(ui, "dnd_two_lists").show_custom(|ui, iter| {
            let mut iter_next = |id: egui::Id, idx: usize, f: IterNextInnerFn| {
                iter.next(ui, id, idx, true, |ui, dnd_item| dnd_item.ui(ui, f));
            };

            // Draw first list
            draw_list(&self.list_1, &mut idx, &mut iter_next);

            // Draw separator
            iter_next(
                egui::Id::new("dnd_two_lists_separator"),
                idx,
                Box::new(
                    |ui: &mut egui::Ui,
                     _handle: hello_egui::dnd::Handle,
                     _item_state: hello_egui::dnd::ItemState| {
                        ui.separator();
                    },
                ),
            );

            // Draw second list
            draw_list(&self.list_2, &mut idx, &mut iter_next);
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
