use crate::dnd::item::Item;

const LIST_INDENT: f32 = 20f32;

/// A structure that allows arbitrary nested lists where any item in the list can be dragged to any
/// other list (except to one of its child lists).
///
/// This method requires structuring the `ui()` method very carefully to avoid recursion within
/// `ItemIterator::next()` and may not work with more complex egui elements.
pub struct NestedLists {
    name: String,
    root: Vec<ItemList>,
}

struct DndIdxBounds {
    start: usize,
    end: usize,
}

impl DndIdxBounds {
    const fn contains(&self, dnd_idx: usize) -> bool {
        dnd_idx >= self.start && dnd_idx <= self.end
    }
}

struct ItemList {
    item: Item,
    children: Vec<ItemList>,

    // dnd state
    dnd_idx: usize,
    dnd_children_idx_bounds: DndIdxBounds, // index of first item and last separator (which may be equal if empty)
    dnd_dragged: bool,
}

impl ItemList {
    fn new(next_id: &mut usize, depth: usize, num_children_per_level: usize) -> Self {
        let item = Item::new(*next_id);
        *next_id += 1;

        let mut children = Vec::<Self>::new();
        if depth > 0 {
            for _ in 0..num_children_per_level {
                children.push(Self::new(next_id, depth - 1, num_children_per_level));
            }
        }

        Self {
            item,
            children,
            dnd_idx: 0,
            dnd_children_idx_bounds: DndIdxBounds { start: 0, end: 0 },
            dnd_dragged: false,
        }
    }

    fn can_insert(&mut self, dnd_idx: usize) -> bool {
        !self.dnd_dragged && self.dnd_children_idx_bounds.contains(dnd_idx)
    }

    fn insert(&mut self, dnd_idx: usize, item: Self) {
        assert!(
            self.can_insert(dnd_idx),
            "Call can_insert() to check if insertion is possible."
        );

        if self.dnd_children_idx_bounds.end == dnd_idx {
            self.children.push(item);
            return;
        }

        for i in 0..self.children.len() {
            let child = &mut self.children[i];

            if child.dnd_idx == dnd_idx {
                self.children.insert(i, item);
                return;
            } else if child.can_insert(dnd_idx) {
                child.insert(dnd_idx, item);
                return;
            }
        }

        panic!("Failed to insert at dnd_idx {dnd_idx}!");
    }

    fn remove_child_item(&mut self, dnd_idx: usize) -> Option<Self> {
        for i in 0..self.children.len() {
            let child = &mut self.children[i];

            if child.dnd_idx == dnd_idx {
                return Some(self.children.remove(i));
            }

            if child.dnd_children_idx_bounds.contains(dnd_idx) {
                return child.remove_child_item(dnd_idx);
            }
        }

        None
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        dnd_item_iter: &mut hello_egui::dnd::item_iterator::ItemIterator,
        dnd_idx: &mut usize,
    ) {
        // Draw item
        self.dnd_idx = *dnd_idx;
        dnd_item_iter.next(
            ui,
            egui::Id::new(self.item.id),
            *dnd_idx,
            true,
            |ui, dnd_item| {
                dnd_item.ui(ui, |ui, handle, item_state| {
                    self.dnd_dragged = item_state.dragged;
                    self.item.ui(ui, handle);

                    if !self.dnd_dragged {
                        ui.horizontal(|ui| {
                            ui.add_space(LIST_INDENT);
                            ui.vertical(|ui| {
                                ui.separator();
                            })
                        });
                    }
                })
            },
        );
        *dnd_idx += 1;

        // Draw children if not being dragged
        if !self.dnd_dragged {
            ui.horizontal(|ui| {
                ui.add_space(LIST_INDENT);
                ui.vertical(|ui| {
                    let start = *dnd_idx;
                    for child_idx in 0..self.children.len() {
                        self.children[child_idx].ui(ui, dnd_item_iter, dnd_idx);
                    }

                    dnd_item_iter.next(
                        ui,
                        egui::Id::new(format!("{}_end_separator", self.item.id)),
                        *dnd_idx,
                        true,
                        |ui, dnd_item| {
                            dnd_item.ui(ui, |ui, _handle, _item_state| {
                                ui.separator();
                            })
                        },
                    );
                    self.dnd_children_idx_bounds = DndIdxBounds {
                        start,
                        end: *dnd_idx,
                    };
                    *dnd_idx += 1;
                });
            });
        }

        // At the end of this function, dnd_idx points to the next item in this list that this
        // ItemList belongs to
    }
}

impl NestedLists {
    pub fn new(
        name: String,
        start_id: usize,
        depth: usize,
        num_items_per_list: usize,
    ) -> (Self, usize) {
        let mut next_id = start_id;

        let mut root = Vec::<ItemList>::new();
        for _ in 0..num_items_per_list {
            root.push(ItemList::new(&mut next_id, depth - 1, num_items_per_list));
        }

        (Self { name, root }, next_id)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut dnd_idx = 0usize;
        let response = hello_egui::dnd::dnd(ui, self.name.as_str()).show_custom(|ui, item_iter| {
            for i in 0..self.root.len() {
                self.root[i].ui(ui, item_iter, &mut dnd_idx);
            }
        });
        ui.separator();

        let dnd_idx = dnd_idx;

        // TODO: avoid duplicating removal and insertion logic here and in ItemList::insert() and ::remove_child_item()
        if let Some(update) = response.update {
            let dnd_idx_start = update.from;
            let dnd_idx_end = update.to;

            // TODO: find ItemList to insert into before doing the actual removal (to skip start and end equality check)
            // TODO: if start and end are in the same ItemList, use shift_vec instead
            if dnd_idx_start != dnd_idx_end {
                let mut removed_item: Option<ItemList> = None;

                for i in 0..self.root.len() {
                    removed_item = if self.root[i].dnd_idx == dnd_idx_start {
                        Some(self.root.remove(i))
                    } else {
                        self.root[i].remove_child_item(dnd_idx_start)
                    };

                    if removed_item.is_some() {
                        break;
                    }
                }

                if let Some(removed_item) = removed_item {
                    if dnd_idx == dnd_idx_end {
                        self.root.push(removed_item);
                    } else {
                        for i in 0..self.root.len() {
                            let child = &mut self.root[i];

                            if child.dnd_idx == dnd_idx_end {
                                self.root.insert(i, removed_item);
                                break;
                            } else if child.can_insert(dnd_idx_end) {
                                child.insert(dnd_idx_end, removed_item);
                                break;
                            }
                        }
                    }
                } else {
                    panic!("Failed to find removed item at idx {dnd_idx_start}!");
                }
            }
        }
    }
}
