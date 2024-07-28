use crate::util::Item;
use std::hash::{Hash, Hasher};

/// Allows drawing nested lists whose items can only be dragged internally (not across list boundaries).
///
/// Only supports nested lists that are 2 deep!
pub struct SeparateNestedLists {
    name: String,
    root: Vec<ItemList>,
}

struct ItemList {
    item: Item,
    children: Option<Vec<ItemList>>,
}

impl Hash for ItemList {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item.id.hash(state);
    }
}

impl SeparateNestedLists {
    pub fn new(name: String, start_id: usize, num_items_per_list: usize) -> (Self, usize) {
        let mut next_id_storage = start_id;

        let mut next_id = || -> usize {
            let result = next_id_storage;
            next_id_storage += 1;
            result
        };

        let mut root = Vec::<ItemList>::new();
        for _ in 0..num_items_per_list {
            let mut children = Vec::<ItemList>::new();

            for _ in 0..num_items_per_list {
                let item = Item::new(next_id());
                children.push(ItemList {
                    item,
                    children: None,
                });
            }

            root.push(ItemList {
                item: Item::new(next_id()),
                children: Some(children),
            });
        }

        (Self { name, root }, next_id())
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        hello_egui::dnd::dnd(ui, self.name.as_str()).show_vec(
            &mut self.root,
            |ui, item_list, handle, _| {
                item_list.item.ui(ui, handle);
                if let Some(children) = &mut item_list.children {
                    ui.indent(format!("{}_indent", item_list.item.id), |ui| {
                        hello_egui::dnd::dnd(ui, item_list.item.id.to_string()).show_vec(
                            children,
                            |ui, item_list, handle, _| {
                                item_list.item.ui(ui, handle);
                            },
                        )
                    });
                }
            },
        );
    }
}
