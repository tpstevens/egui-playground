use crate::util::Item;

/// A structure that contains multiple lists (separated with a horizontal line) whose items can be
/// dragged freely between them.
pub struct MultipleLists {
    lists: Vec<ItemList>,

    // dnd state (stored here to avoid allocation on every ui())
    lists_dnd_idx_bounds: Vec<ListDndIdxBounds>,
}

struct ItemList {
    id: String,
    items: Vec<Item>,
}

impl ItemList {
    /// Draws the list, updating the dnd index bounds that correspond to the list's start and end.
    ///
    /// This example assumes that each sub-structure of `MultipleLists` is only one list (and thus
    /// it's reasonable to update a single `ListDndIdxBounds` structure). If `ItemList` was a more
    /// complex structure, `ui()` would need to take a reference to
    /// `MultipleLists::lists_dnd_idx_bounds` and a mutable reference to some `list_idx` count that
    /// could be used to track which sub-lists belong to which struct. That might also make it
    /// challenging to handle construction of `lists_dnd_idx_bounds` -- for really dynamic
    /// structures, it would have to be created and filled out on each frame.
    fn ui(
        &self,
        ui: &mut egui::Ui,
        dnd_item_iter: &mut hello_egui::dnd::item_iterator::ItemIterator,
        dnd_idx: &mut usize,
        draw_header: bool,
        bounds: &mut ListDndIdxBounds,
    ) {
        if draw_header {
            dnd_item_iter.next(
                ui,
                egui::Id::new(format!("separator_{}", self.id)),
                *dnd_idx,
                true,
                |ui, dnd_item| {
                    dnd_item.ui(ui, |ui, _handle, _item_state| {
                        ui.separator();
                        self.draw_header(ui);
                    })
                },
            );

            *dnd_idx += 1;
        }

        bounds.start = *dnd_idx;
        for item in &self.items {
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
        bounds.end = *dnd_idx - 1;
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.label(self.id.as_str());
    }
}

/// Pairs a list index and an item index (relative to that list, not the overall dnd index).
struct ListAndItemIdx {
    list_index: usize,
    list_internal_index: usize,
}

/// Contains the start and end indices of a list in terms of the dnd index (the value associated
/// with each call to `egui_dnd::dnd::show_custom()`'s iterator)
struct ListDndIdxBounds {
    start: usize,
    end: usize,
}

impl MultipleLists {
    pub fn new(num_lists: usize, start_id: usize, num_items_per_list: usize) -> Self {
        let mut lists = Vec::<ItemList>::new();
        let mut lists_dnd_idx_bounds = Vec::<ListDndIdxBounds>::new();

        for i in 0..num_lists {
            let mut items = Vec::<Item>::new();
            let offset = start_id + i * num_items_per_list;
            for j in 0..num_items_per_list {
                items.push(Item::new(offset + j));
            }

            lists.push(ItemList {
                id: format!("multi_list_{i}"),
                items,
            });
            lists_dnd_idx_bounds.push(ListDndIdxBounds { start: 0, end: 0 });
        }

        Self {
            lists,
            lists_dnd_idx_bounds,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut idx = 0usize;

        // Draw the first header outside the list to ensure that nothing can be dragged before it
        self.lists[0].draw_header(ui);

        let response = hello_egui::dnd::dnd(ui, "dnd_multiple_lists").show_custom(|ui, iter| {
            for i in 0..self.lists.len() {
                self.lists[i].ui(ui, iter, &mut idx, i > 0, &mut self.lists_dnd_idx_bounds[i]);
            }
        });

        if let Some(update) = response.update {
            let from = convert_dnd_idx_to_list_and_item_idx(
                &self.lists_dnd_idx_bounds,
                update.from,
                false,
            );
            let to =
                convert_dnd_idx_to_list_and_item_idx(&self.lists_dnd_idx_bounds, update.to, true);

            if let (Some(from), Some(to)) = (from, to) {
                if from.list_index == to.list_index {
                    hello_egui::dnd::utils::shift_vec(
                        from.list_internal_index,
                        to.list_internal_index,
                        &mut self.lists[from.list_index].items,
                    );
                } else {
                    let removed_item = self.lists[from.list_index]
                        .items
                        .remove(from.list_internal_index);
                    self.lists[to.list_index]
                        .items
                        .insert(to.list_internal_index, removed_item);
                }
            }
        }
    }
}

fn convert_dnd_idx_to_list_and_item_idx(
    lists_dnd_idx_bounds: &[ListDndIdxBounds],
    dnd_idx: usize,
    for_insertion: bool,
) -> Option<ListAndItemIdx> {
    for list_idx in 0..lists_dnd_idx_bounds.len() {
        let dnd_idx_start = lists_dnd_idx_bounds[list_idx].start;
        let dnd_idx_end = lists_dnd_idx_bounds[list_idx].end;

        if dnd_idx >= dnd_idx_start && dnd_idx <= dnd_idx_end {
            return Some(ListAndItemIdx {
                list_index: list_idx,
                list_internal_index: dnd_idx - dnd_idx_start,
            });
        }

        if for_insertion {
            // If reaches this point, dnd_index must be *after* this list's bounds. Append the item to
            // this list if the list is the last one or if the item falls before the next list's start.
            if list_idx == lists_dnd_idx_bounds.len() - 1
                || dnd_idx < lists_dnd_idx_bounds[list_idx + 1].start
            {
                return Some(ListAndItemIdx {
                    list_index: list_idx,
                    list_internal_index: dnd_idx - dnd_idx_start,
                });
            }
        }
    }

    None
}