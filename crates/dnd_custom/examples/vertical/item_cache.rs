use dnd_custom::generic_list;
use dnd_custom::generic_list::DndState;
use std::collections::HashMap;

type ItemId = usize;
type ListId = usize;

struct Item {
    /// Must be unique among item IDs (but not among other types of IDs, like list IDs).
    id: ItemId,
    /// Item title (like the title of an ADO ticket).
    title: String,
    children: ListId,

    // ui state
    ui_collapsed: bool,
}

impl Item {
    fn new(id: ItemId, title: &str, children: ListId) -> Self {
        Self {
            id,
            title: title.to_string(),
            children,
            ui_collapsed: false,
        }
    }
}

struct ItemList {
    /// Must be unique among list IDs (but not among other types of IDs, like item IDs).
    id: ListId,
    /// Describes how the contained set of items is sorted by the user (priority, dependency, etc.).
    sorted_by: String,
    data: Vec<ItemId>,
}

/// Stores items and lists and implements the drawing interface required by `nested_list::DndItemCache`.
pub struct ItemCache {
    items: HashMap<ItemId, Item>,
    lists: HashMap<ListId, ItemList>,
    root_list_id: ListId,
}

impl ItemCache {
    /// Generates a demo set of nested items.
    pub fn demo_new(next_id: usize, depth: usize, list_len: usize) -> Self {
        let mut items = HashMap::<ItemId, Item>::new();
        let mut lists = HashMap::<ListId, ItemList>::new();

        let mut next_id = next_id;

        let root_list_id = Self::new_list(&mut next_id, &mut items, &mut lists, depth, list_len);

        Self {
            items,
            lists,
            root_list_id,
        }
    }

    fn new_list(
        next_id: &mut usize,
        items: &mut HashMap<ItemId, Item>,
        lists: &mut HashMap<ListId, ItemList>,
        depth: usize,
        list_len: usize,
    ) -> ListId {
        let list_id = *next_id;
        *next_id += 1;

        let mut data = Vec::<ItemId>::new();
        if depth > 0 {
            for _ in 0..list_len {
                let item_id: ItemId = *next_id;
                *next_id += 1;

                let item = Item::new(
                    item_id,
                    "placeholder title",
                    Self::new_list(next_id, items, lists, depth - 1, list_len),
                );
                items.insert(item_id, item);
                data.push(item_id);
            }
        }

        lists.insert(
            list_id,
            ItemList {
                id: list_id,
                sorted_by: String::new(),
                data,
            },
        );
        list_id
    }

    pub const fn root_list_id(&self) -> ItemId {
        self.root_list_id
    }
}

impl generic_list::DndItemCache for ItemCache {
    type ItemId = ItemId;
    type ListId = ListId;

    fn get_child_list_id(&self, item_id: &Self::ItemId) -> Option<Self::ListId> {
        self.items.get(item_id).map(|item| item.children)
    }

    fn ui_item(
        &mut self,
        item_id: &Self::ItemId,
        ui: &mut egui::Ui,
        handle: hello_egui::dnd::Handle,
        force_collapsed: bool,
    ) -> bool {
        if let Some(item) = self.items.get_mut(item_id) {
            let collapsed_any = item.ui_collapsed || force_collapsed;

            // TODO: allocate elements to ensure that handle and sorted by don't overlap
            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    if collapsed_any {
                        ui.toggle_value(&mut item.ui_collapsed, ">");
                    } else {
                        ui.toggle_value(&mut item.ui_collapsed, "v");
                    }
                    ui.label(format!("({}) {}", item.id, item.title));
                });

                if !collapsed_any {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let list_id = item.children;
                        match self.lists.get_mut(&list_id) {
                            Some(list) => {
                                ui.horizontal(|ui| {
                                    // TODO: constrain the length of editor
                                    ui.text_edit_singleline(&mut list.sorted_by);
                                    ui.label("Sorted by: ");
                                });
                            }
                            None => {
                                ui.label(format!("List {list_id} not found!"));
                            }
                        }
                    });
                }
            });

            collapsed_any
        } else {
            ui.label(format!("Item {item_id} not found!"));
            false
        }
    }

    fn ui_list_header(
        &mut self,
        _list_id: &Self::ListId,
        config: &generic_list::UiListHeaderConfig,
        ui: &mut egui::Ui,
    ) {
        if matches!(config, generic_list::UiListHeaderConfig::Root) {
            ui.label("[placeholder root header]");
        }
    }

    fn ui_list_contents(
        &mut self,
        list_id: &Self::ListId,
        config: &generic_list::UiListConfig,
        ui: &mut egui::Ui,
        dnd_state: &mut DndState,
        ui_items: impl FnOnce(&mut Self, &mut egui::Ui, &mut DndState, &Vec<Self::ItemId>),
        ui_footer: impl FnOnce(&mut egui::Ui, &mut DndState, Box<dyn FnOnce(&mut egui::Ui)>),
    ) {
        if let Some(list) = self.lists.get(list_id) {
            let clone = list.data.clone();
            ui.vertical(|ui| match config {
                generic_list::UiListConfig::Root => {
                    ui_items(self, ui, dnd_state, &clone);
                    ui_footer(
                        ui,
                        dnd_state,
                        Box::new(|ui| {
                            ui.label("[placeholder root footer]");
                        }),
                    );
                }
                generic_list::UiListConfig::SubList(cfg) => {
                    if cfg.draw_header {
                        self.ui_list_header(
                            list_id,
                            &generic_list::UiListHeaderConfig::SubList,
                            ui,
                        );
                    }

                    ui.indent(egui::Id::new(list_id).with("indent"), |ui| {
                        ui_items(self, ui, dnd_state, &clone);
                        ui_footer(
                            ui,
                            dnd_state,
                            Box::new(|ui| {
                                ui.separator();
                            }),
                        );
                    });
                }
            });
        } else {
            ui.label(format!("List {list_id} not found!"));
        }
    }
}
