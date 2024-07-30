use dnd_custom::ghl;
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

/// Stores items and lists and implements the drawing interface required by `ghl::Ghl`.
pub struct ItemCache {
    items: HashMap<ItemId, Item>,
    lists: HashMap<ListId, ItemList>,
    root_list_id: ListId,
}

impl ItemCache {
    /// Generates a demo set of nested items.
    pub fn demo_new(depth: usize, list_len: usize) -> Self {
        let mut items = HashMap::<ItemId, Item>::new();
        let mut lists = HashMap::<ListId, ItemList>::new();

        let mut next_list_id = 0usize;
        let mut next_item_id = 0usize;

        let root_list_id = Self::new_list(
            &mut next_list_id,
            &mut next_item_id,
            &mut items,
            &mut lists,
            depth,
            list_len,
        );

        Self {
            items,
            lists,
            root_list_id,
        }
    }

    pub fn handle_update(&mut self, update: &ghl::DragUpdate<ListId>) -> bool {
        if let Some(removed_item) = self
            .lists
            .get_mut(&update.from.list_id)
            .map(|list| list.data.remove(update.from.idx))
        {
            match &update.to {
                ghl::DragDestination::Insert(to) => {
                    if let Some(list) = self.lists.get_mut(&to.list_id) {
                        list.data.insert(to.idx, removed_item);
                        return true;
                    }
                }
                ghl::DragDestination::Push(to) => {
                    if let Some(list) = self.lists.get_mut(to) {
                        list.data.push(removed_item);
                        return true;
                    }
                }
                ghl::DragDestination::Within(to) => {
                    if let Some(list) = self.lists.get_mut(&update.from.list_id) {
                        // TODO: rotate list for efficiency instead of removing and inserting
                        list.data.insert(*to, removed_item);
                        return true;
                    }
                }
            }
        }

        false
    }

    fn new_list(
        next_list_id: &mut usize,
        next_item_id: &mut usize,
        items: &mut HashMap<ItemId, Item>,
        lists: &mut HashMap<ListId, ItemList>,
        depth: usize,
        list_len: usize,
    ) -> ListId {
        let list_id = *next_list_id;
        *next_list_id += 1;

        let mut data = Vec::<ItemId>::new();
        if depth > 0 {
            for _ in 0..list_len {
                let item_id: ItemId = *next_item_id;
                *next_item_id += 1;

                let item = Item::new(
                    item_id,
                    "placeholder title",
                    Self::new_list(
                        next_list_id,
                        next_item_id,
                        items,
                        lists,
                        depth - 1,
                        list_len,
                    ),
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

impl ghl::Ghl for ItemCache {
    type ItemId = ItemId;
    type ListId = ListId;

    fn get_child_list_id(&self, item_id: &Self::ItemId) -> Option<Self::ListId> {
        self.items.get(item_id).map(|item| item.children)
    }

    fn get_list_contents(&self, list_id: &Self::ListId) -> Option<&Vec<Self::ItemId>> {
        self.lists
            .get(list_id)
            .map(|item_list| item_list.data.as_ref())
    }

    fn ui_item(
        &mut self,
        item_id: &Self::ItemId,
        ui: &mut egui::Ui,
        handle: hello_egui::dnd::Handle,
        force_collapsed: bool,
    ) -> bool {
        if let Some(item) = self.items.get_mut(item_id) {
            let mut collapsed_any = item.ui_collapsed || force_collapsed;

            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    if collapsed_any {
                        ui.toggle_value(&mut item.ui_collapsed, ">");
                    } else {
                        ui.toggle_value(&mut item.ui_collapsed, "v");
                    }
                    collapsed_any |= item.ui_collapsed;

                    ui.label(format!(
                        "(item = {}) {} (list = {})",
                        item.id, item.title, item.children
                    ));
                });

                if !collapsed_any {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let list_id = item.children;
                        match self.lists.get_mut(&list_id) {
                            Some(list) => {
                                ui.horizontal(|ui| {
                                    // TODO: constrain the length of TextEdit so it doesn't overlap the handle
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
        config: &ghl::UiListHeaderConfig,
        ui: &mut egui::Ui,
    ) {
        if matches!(config, ghl::UiListHeaderConfig::Root) {
            ui.label("[placeholder root header]");
        }
    }

    fn ui_list_contents(
        &mut self,
        list_id: &Self::ListId,
        config: &ghl::UiListConfig,
        ui: &mut egui::Ui,
        ui_state: &mut ghl::UiState<Self::ItemId, Self::ListId>,
        ui_items: impl FnOnce(&mut Self, &mut egui::Ui, &mut ghl::UiState<Self::ItemId, Self::ListId>),
        ui_footer: impl FnOnce(
            &mut egui::Ui,
            &mut ghl::UiState<Self::ItemId, Self::ListId>,
            Box<dyn FnOnce(&mut egui::Ui)>,
        ),
    ) {
        ui.vertical(|ui| match config {
            ghl::UiListConfig::Root => {
                ui_items(self, ui, ui_state);
                ui_footer(
                    ui,
                    ui_state,
                    Box::new(|ui| {
                        ui.label("[placeholder root footer]");
                    }),
                );
            }
            ghl::UiListConfig::SubList(cfg) => {
                if cfg.draw_header {
                    self.ui_list_header(list_id, &ghl::UiListHeaderConfig::SubList, ui);
                }

                ui.indent(egui::Id::new(list_id).with("indent"), |ui| {
                    ui_items(self, ui, ui_state);
                    ui_footer(
                        ui,
                        ui_state,
                        Box::new(|ui| {
                            ui.separator();
                        }),
                    );
                });
            }
        });
    }
}
