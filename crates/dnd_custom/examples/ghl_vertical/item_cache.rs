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

    // ui state
    ui_outlined: bool,
}

impl ItemList {
    fn new(id: ListId) -> Self {
        Self {
            id,
            sorted_by: String::new(),
            data: Vec::<ItemId>::new(),
            ui_outlined: false,
        }
    }
}

/// Stores items and lists and implements the drawing interface required by `ghl::Ghl`.
pub struct ItemCache {
    items: HashMap<ItemId, Item>,
    next_item_id: ItemId,
    lists: HashMap<ListId, ItemList>,
    next_list_id: ListId,
    root_list_id: ListId,

    // Temporary/UI state
    ui_modal_dialog: Option<ModalDialog>,
    ui_selected_item: Option<ItemId>,
}

enum ListPosition {
    Start,
    End,
    At(usize),
}

#[derive(Clone)]
struct DeleteItemConfirmation {
    item: ItemId,
    parent_list: ItemId,
}

#[derive(Clone)]
enum ModalDialog {
    DeleteItemConfirmation(DeleteItemConfirmation),
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
            next_item_id,
            lists,
            next_list_id,
            root_list_id,
            ui_modal_dialog: None,
            ui_selected_item: None,
        }
    }

    pub const fn modal_activated(&self) -> bool {
        self.ui_modal_dialog.is_some()
    }

    // TODO: define reusable modal dialog that accepts a width and height, question text, and answer text and returns whether confirmation was clicked
    // TODO: calculate size of elements in sizing pass first (similar to https://github.com/emilk/egui/pull/4612/)
    pub fn handle_modal(&mut self, ui: &mut egui::Ui) {
        if let Some(modal) = self.ui_modal_dialog.clone() {
            // If modal is shown, then current panel's UI has been disabled
            ui.add_enabled_ui(true, |ui| {
                let parent_rect = ui.clip_rect();
                let parent_center = parent_rect.center();

                ui.allocate_ui_at_rect(parent_rect, |ui| match modal {
                    ModalDialog::DeleteItemConfirmation(confirmation) => {
                        let dialog_width = 200f32;
                        let dialog_height = 100f32;

                        let top_left = egui::Pos2::new(
                            parent_center.x - dialog_width / 2.0,
                            parent_center.y - dialog_height / 2.0,
                        );

                        let dialog_rect = egui::Rect::from_min_size(
                            top_left,
                            egui::Vec2::new(dialog_width, dialog_height),
                        );

                        ui.allocate_ui_at_rect(dialog_rect, |ui| {
                            ui.vertical_centered(|ui| {
                                egui::Frame::default()
                                    .stroke(egui::Stroke::new(1f32, egui::Color32::LIGHT_GRAY))
                                    .fill(egui::Color32::BLACK)
                                    .inner_margin(ui.style().spacing.window_margin)
                                    .show(ui, |ui| {
                                        ui.label("Deleting this item will also delete its children. Proceed?");

                                        ui.columns(2, |columns| {
                                            columns[0].allocate_ui_with_layout(
                                                egui::Vec2::new(0f32, 0f32),
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("Cancel").clicked() {
                                                        self.ui_modal_dialog = None;
                                                    }
                                                },
                                            );

                                            columns[1].allocate_ui_with_layout(
                                                egui::Vec2::new(0f32, 0f32),
                                                egui::Layout::left_to_right(egui::Align::TOP),
                                                |ui| {
                                                    if ui.button("Confirm").clicked() {
                                                        self.remove_item_from_parent_and_delete(
                                                            confirmation.parent_list,
                                                            confirmation.item,
                                                        );
                                                        self.ui_modal_dialog = None;
                                                    }
                                                },
                                            );
                                        });
                                    });
                            });
                        });
                    }
                });
            });
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

    fn add_item(&mut self, list_id: ListId, pos: &ListPosition) {
        let child_list_id = self.next_list_id;
        self.lists
            .insert(child_list_id, ItemList::new(child_list_id));

        let child_id = self.next_item_id;
        self.items
            .insert(child_id, Item::new(child_list_id, "", child_list_id));

        if let Some(list) = self.lists.get_mut(&list_id) {
            match pos {
                ListPosition::Start => {
                    list.data.insert(0, child_id);
                }
                ListPosition::End => {
                    list.data.push(child_id);
                }
                ListPosition::At(idx) => {
                    list.data.insert(*idx, child_id);
                }
            }
            self.next_list_id += 1;
            self.next_item_id += 1;
        } else {
            self.lists.remove(&child_list_id);
            self.items.remove(&child_id);
        }
    }

    /// Remove an item from a parent list, then delete the item and its children.
    fn remove_item_from_parent_and_delete(&mut self, list_id: ListId, item_id: ItemId) {
        if let Some(list) = self.lists.get_mut(&list_id) {
            if let Some(item_idx) = list.data.iter().position(|id| *id == item_id) {
                list.data.remove(item_idx);
            }
        } else {
            println!("couldn't find list {list_id}");
        }

        self.delete_item_and_children(item_id);
    }

    /// Delete an item and its children.
    fn delete_item_and_children(&mut self, item_id: ItemId) {
        // Remove child list
        if let Some(child_list_id) = self.items.get(&item_id).map(|item| item.children) {
            self.remove_list(child_list_id);
        }

        // Remove item
        self.items.remove(&item_id);
    }

    /// Recursively remove a list and its contents.
    ///
    /// **Note: Assumes that no item references this list!**
    fn remove_list(&mut self, list_id: ListId) {
        let mut items_to_remove = Vec::<ItemId>::new();

        if let Some(list) = self.lists.get_mut(&list_id) {
            items_to_remove = std::mem::take(&mut list.data);
        }

        self.lists.remove(&list_id);
        for item_id in items_to_remove {
            self.delete_item_and_children(item_id);
        }
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
                ui_outlined: false,
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

    fn get_child_list_id(&self, item_id: Self::ItemId) -> Option<Self::ListId> {
        self.items.get(&item_id).map(|item| item.children)
    }

    fn get_list_contents(
        &self,
        list_id: Self::ListId,
    ) -> Option<impl Iterator<Item = Self::ItemId>> {
        self.lists
            .get(&list_id)
            .map(|item_list| item_list.data.iter().copied())
    }

    // TODO: refactor
    #[allow(clippy::too_many_lines)]
    fn ui_item(
        &mut self,
        item_id: Self::ItemId,
        parent_list_id: Self::ListId,
        idx_in_parent_list: usize,
        ui: &mut egui::Ui,
        handle: hello_egui::dnd::Handle,
        force_collapsed: bool,
    ) -> (bool, bool) {
        let mut add_item_to_list: Option<(ListId, ListPosition)> = None;
        let mut delete_item_from_list: Option<(ListId, ItemId)> = None;
        let mut collapsed_any = false;
        let mut modal_activated = self.modal_activated();

        if let Some(item) = self.items.get_mut(&item_id) {
            collapsed_any = item.ui_collapsed || force_collapsed;

            let stroke = if self
                .ui_selected_item
                .map_or_else(|| false, |id| id == item_id)
            {
                egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY)
            } else {
                egui::Stroke::NONE
            };

            let mut draw_ctx_menu = |ui_selected_item: &mut Option<ItemId>, ui: &mut egui::Ui| {
                *ui_selected_item = Some(item_id);

                if ui.button("Add item above").clicked() {
                    add_item_to_list = Some((parent_list_id, ListPosition::At(idx_in_parent_list)));
                    ui.close_menu();
                }

                if ui.button("Add item below").clicked() {
                    add_item_to_list =
                        Some((parent_list_id, ListPosition::At(idx_in_parent_list + 1)));
                    ui.close_menu();
                }

                if ui.button("Add child item at start").clicked() {
                    add_item_to_list = Some((item.children, ListPosition::Start));
                    ui.close_menu();
                };

                if ui.button("Add child item at end").clicked() {
                    add_item_to_list = Some((item.children, ListPosition::End));
                    ui.close_menu();
                }
            };

            ui.push_id(item_id, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(1f32))
                    .stroke(stroke)
                    .show(ui, |ui| {
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

                            if collapsed_any {
                                ui.allocate_space(ui.available_size());
                            } else {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let list_id = item.children;
                                        match self.lists.get_mut(&list_id) {
                                            Some(list) => {
                                                if ui.button("x").clicked() {
                                                    if list.data.is_empty() {
                                                        delete_item_from_list =
                                                            Some((parent_list_id, item_id));
                                                    } else {
                                                        self.ui_selected_item = Some(item_id);
                                                        self.ui_modal_dialog = Some(
                                                            ModalDialog::DeleteItemConfirmation(
                                                                DeleteItemConfirmation {
                                                                    item: item_id,
                                                                    parent_list: parent_list_id,
                                                                },
                                                            ),
                                                        );
                                                        modal_activated = true;
                                                    }
                                                }

                                                ui.add_sized(
                                                    egui::Vec2::new(200f32, ui.available_size().y),
                                                    |ui: &mut egui::Ui| {
                                                        ui.text_edit_singleline(&mut list.sorted_by)
                                                    },
                                                );

                                                let sorted_by_label_rect =
                                                    ui.label("Sorted by: ").interact_rect;

                                                // Capture right-clicks on "Sorted by" label
                                                ui.interact(
                                                    sorted_by_label_rect,
                                                    egui::Id::new("label_sorted_by").with(item_id),
                                                    egui::Sense::click(),
                                                )
                                                .context_menu(|ui| {
                                                    draw_ctx_menu(&mut self.ui_selected_item, ui);
                                                });
                                            }
                                            None => {
                                                ui.label(format!("List {list_id} not found!"));
                                            }
                                        }
                                    },
                                );
                            }
                        })
                        .response
                        .context_menu(|ui| draw_ctx_menu(&mut self.ui_selected_item, ui));
                    });

                if !ui.ctx().is_context_menu_open() && !modal_activated {
                    self.ui_selected_item = None;
                }
            });
        } else {
            ui.label(format!("Item {item_id} not found!"));
        }

        if let Some((list_id, pos)) = add_item_to_list {
            self.add_item(list_id, &pos);
        }

        let mut item_deleted = false;
        if let Some((list_id, item_id)) = delete_item_from_list {
            item_deleted = true;
            self.remove_item_from_parent_and_delete(list_id, item_id);
        }

        (collapsed_any, item_deleted)
    }

    fn ui_list_header(
        &mut self,
        list_id: Self::ListId,
        config: &ghl::UiListHeaderConfig,
        ui: &mut egui::Ui,
    ) {
        if matches!(config, ghl::UiListHeaderConfig::Root) {
            if let Some(list) = self.lists.get_mut(&list_id) {
                let mut add_item_to_list: Option<ListPosition> = None;

                let draw_ctx_menu = |list: &mut ItemList,
                                     add_item: &mut Option<ListPosition>,
                                     ui: &mut egui::Ui| {
                    list.ui_outlined = true;

                    if ui.button("Add item to start").clicked() {
                        *add_item = Some(ListPosition::Start);
                        ui.close_menu();
                    };

                    if ui.button("Add item to end").clicked() {
                        *add_item = Some(ListPosition::End);
                        ui.close_menu();
                    }
                };

                let stroke = if list.ui_outlined {
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY)
                } else {
                    egui::Stroke::NONE
                };

                egui::Frame::none()
                    .inner_margin(egui::Margin::same(1f32))
                    .stroke(stroke)
                    .show(ui, |ui| {
                        // Capture right-clicks on frame
                        ui.interact(
                            ui.clip_rect(),
                            egui::Id::new("list_header_frame").with(list_id),
                            egui::Sense::click(),
                        )
                        .context_menu(|ui| {
                            draw_ctx_menu(list, &mut add_item_to_list, ui);
                        });

                        ui.horizontal(|ui| {
                            let button_response = ui.button("+");
                            if button_response.clicked() {
                                add_item_to_list = Some(ListPosition::Start);
                            } else {
                                // Capture right-clicks on button
                                button_response.context_menu(|ui| {
                                    draw_ctx_menu(list, &mut add_item_to_list, ui);
                                });
                            }

                            let label_rect = ui
                                .add(egui::Label::new("[placeholder root header]"))
                                .interact_rect;

                            // Capture right-clicks on label
                            ui.interact(
                                label_rect,
                                egui::Id::new("list_header_label").with(list_id),
                                egui::Sense::click(),
                            )
                            .context_menu(|ui| draw_ctx_menu(list, &mut add_item_to_list, ui));

                            ui.allocate_space(ui.available_size());
                        });
                    });

                if !ui.ctx().is_context_menu_open() {
                    list.ui_outlined = false;
                }

                if let Some(pos) = add_item_to_list {
                    self.add_item(list_id, &pos);
                }
            }
        }
    }

    fn ui_list_contents(
        &mut self,
        list_id: Self::ListId,
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
