use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

// =================================================================================================
// Public types, traits, and functions

pub enum DragDestination<ListId>
where
    ListId: Copy + Debug + Display + Eq + Hash,
{
    /// Insert into the other list at the given location
    Insert(DragLocation<ListId>),

    /// Push onto the other list
    Push(ListId),

    /// Move within the starting list
    Within(usize),
}

pub struct DragLocation<ListId>
where
    ListId: Copy + Debug + Display + Eq + Hash,
{
    pub list_id: ListId,
    pub idx: usize,
}

pub struct DragUpdate<ListId>
where
    ListId: Copy + Debug + Display + Eq + Hash,
{
    pub from: DragLocation<ListId>,
    pub to: DragDestination<ListId>,
}

pub struct UiSubListConfig {
    pub draw_header: bool,
}

/// Configures how to draw the items in the list
pub enum UiListConfig {
    Root,
    SubList(UiSubListConfig),
}

/// Configures how to draw the list header
pub enum UiListHeaderConfig {
    Root,
    SubList,
}

/// State that must be passed into closures executed by `Ghl` functions
pub struct UiState<'a, 'b, ItemId, ListId>
where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    pub dnd_idx: &'a mut usize, // public to help with debugging
    item_iter: &'a mut hello_egui::dnd::item_iterator::ItemIterator<'b>,
    item_states: &'a mut HashMap<ItemId, ItemState>,
    list_bounds: &'a mut HashMap<ListId, DndIdxBounds>,
}

/// Implementing this trait for a "generic hierarchical list" of items (e.g. a list of items with
/// lists) makes it possible to render them in a drag-and-drop `hello_egui::dnd` container.
/// `hello_egui::dnd` currently does not support nested lists out of the box, so drawing is split
/// into a few methods to work around this restriction.
///
/// Drawing starts by calling [`ui_list_contents()`]. The `config` parameter indicates whether the
/// current list is the root, and if the current list is not the root, whether the list header
/// should be drawn.
///
/// The list header must be drawable separately from the list contents because sometimes it must be
/// drawn with its parent item in `hello_egui::dnd::item_iterator::ItemIterator::show()` instead
/// of standalone. This ensures that no items can be dragged between the parent item and the list
/// header.
///
/// **Ensure that any header layout modifications (indentation, etc.) are applied in
/// [`ui_list_header()`] instead of in [`ui_list_contents()`] so that both cases are handled
/// similarly**.
///
/// Note that it is reasonable to not draw a list header by leaving [`ui_list_header()`] empty.
/// However, it is **necessary** to draw a list footer in order to disambiguate the ends of lists
/// when dragging items.
// TODO: summary here, details on each function
pub trait Ghl {
    type ItemId: Copy + Debug + Display + Eq + Hash;
    type ListId: Copy + Debug + Display + Eq + Hash;

    /// Get the list that belongs to the given item.
    fn get_child_list_id(&self, item_id: &Self::ItemId) -> Option<Self::ListId>;

    /// Get all the items that belong to the given list.
    // TODO: consider iterator to avoid clone, or at least a slice to avoid Vec
    fn get_list_contents(&self, item_id: &Self::ListId) -> Option<&Vec<Self::ItemId>>;

    /// Draw the item.
    fn ui_item(
        &mut self,
        item_id: &Self::ItemId,
        ui: &mut egui::Ui,
        handle: hello_egui::dnd::Handle,
        force_collapsed: bool,
    ) -> bool;

    /// Draw the header.
    fn ui_list_header(
        &mut self,
        list_id: &Self::ListId,
        config: &UiListHeaderConfig,
        ui: &mut egui::Ui,
    );

    /// In order:
    /// - Draw the header, if requested. Use [`ui_list_header()`] to apply consistent formatting!
    /// - Construct the `egui::Ui` for the list contents (e.g. create an indented vertical layout)
    ///   and call the `ui_items()` function parameter with a list of item IDs.
    /// - Construct the `egui::Ui` for the list footer and call the `ui_footer()` function parameter
    ///   with a closure to draw the footer.
    fn ui_list_contents(
        &mut self,
        list_id: &Self::ListId,
        config: &UiListConfig,
        ui: &mut egui::Ui,
        ui_state: &mut UiState<Self::ItemId, Self::ListId>,
        ui_items: impl FnOnce(&mut Self, &mut egui::Ui, &mut UiState<Self::ItemId, Self::ListId>),
        ui_footer: impl FnOnce(
            &mut egui::Ui,
            &mut UiState<Self::ItemId, Self::ListId>,
            Box<dyn FnOnce(&mut egui::Ui)>,
        ),
    );
}

pub fn ui<ItemId, ListId>(
    ui: &mut egui::Ui,
    ui_id: egui::Id,
    ghl: &mut impl Ghl<ItemId = ItemId, ListId = ListId>,
    root_list_id: &ListId,
) -> Option<DragUpdate<ListId>>
where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    let mut dnd_idx = 0usize;
    let mut list_bounds = HashMap::<ListId, DndIdxBounds>::new();
    let mut item_states = HashMap::<ItemId, ItemState>::new();

    ghl.ui_list_header(root_list_id, &UiListHeaderConfig::Root, ui);
    let response = hello_egui::dnd::dnd(ui, ui_id).show_custom(|ui, item_iter| {
        let mut ui_state = UiState {
            dnd_idx: &mut dnd_idx,
            item_iter,
            list_bounds: &mut list_bounds,
            item_states: &mut item_states,
        };

        draw_list(ghl, root_list_id, &UiListConfig::Root, ui, &mut ui_state);
    });

    if let Some(update) = response.update {
        if update.from != update.to {
            if let Some(from) =
                find_start(root_list_id, update.from, &list_bounds, &item_states, ghl)
            {
                if let Some(to) = find_end(
                    root_list_id,
                    &from.list_id,
                    update.to,
                    &list_bounds,
                    &item_states,
                    ghl,
                ) {
                    return Some(DragUpdate { from, to });
                }
            }
        }
    }

    None
}

// =================================================================================================
// Private types, traits, and functions

#[derive(Debug)]
struct DndIdxBounds {
    start: usize,
    end: usize,
}

impl DndIdxBounds {
    const fn contains(&self, dnd_idx: usize) -> bool {
        dnd_idx >= self.start && dnd_idx <= self.end
    }
}

#[derive(Debug)]
struct ItemState {
    collapsed: bool, // mirrors value stored in Ghl (if any) in order to drag items over collapsed items
    dnd_idx: usize,
    dragging: bool,
}

fn draw_list<ItemId, ListId, T: Ghl<ItemId = ItemId, ListId = ListId>>(
    ghl: &mut T,
    list_id: &ListId,
    config: &UiListConfig,
    ui: &mut egui::Ui,
    ui_state: &mut UiState<ItemId, ListId>,
) where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    let ui_items = |ghl: &mut T, ui: &mut egui::Ui, ui_state: &mut UiState<ItemId, ListId>| {
        if let Some(items) = ghl.get_list_contents(list_id) {
            for item in items.clone() {
                draw_item(ghl, &item, ui, ui_state);
            }
        } else {
            ui.label(
                egui::RichText::new(format!("No items found for list {list_id}"))
                    .color(egui::Color32::RED),
            );
        }
    };

    let ui_footer = |ui: &mut egui::Ui,
                     ui_state: &mut UiState<ItemId, ListId>,
                     footer: Box<dyn FnOnce(&mut egui::Ui)>| {
        match config {
            UiListConfig::Root => {
                // Don't include the footer in the root list to prevent items from being dragged past it
                footer(ui);
            }
            UiListConfig::SubList(_) => {
                ui_state.item_iter.next(
                    ui,
                    egui::Id::new(list_id).with("footer"),
                    *ui_state.dnd_idx,
                    true,
                    |ui, dnd_item| {
                        dnd_item.ui(ui, |ui, _handle, _item_state| {
                            footer(ui);
                        })
                    },
                );
            }
        }
    };

    let start = *ui_state.dnd_idx;
    ghl.ui_list_contents(list_id, config, ui, ui_state, ui_items, ui_footer);
    ui_state.list_bounds.insert(
        *list_id,
        DndIdxBounds {
            start,
            end: *ui_state.dnd_idx,
        },
    );
    *ui_state.dnd_idx += 1;
}

fn draw_item<ItemId, ListId, T: Ghl<ItemId = ItemId, ListId = ListId>>(
    ghl: &mut T,
    item_id: &ItemId,
    ui: &mut egui::Ui,
    ui_state: &mut UiState<ItemId, ListId>,
) where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    if let Some(list_id) = ghl.get_child_list_id(item_id) {
        let mut item_dragging = false;
        let mut item_collapsed = false;
        let item_idx = *ui_state.dnd_idx;

        ui_state.item_iter.next(
            ui,
            egui::Id::new(item_id),
            *ui_state.dnd_idx,
            true,
            |ui, dnd_item| {
                dnd_item.ui(ui, |ui, handle, item_state| {
                    item_dragging = item_state.dragged;
                    item_collapsed = ghl.ui_item(item_id, ui, handle, item_dragging);
                    if !item_collapsed && !item_dragging {
                        ghl.ui_list_header(&list_id, &UiListHeaderConfig::SubList, ui);
                    }
                })
            },
        );

        ui_state.item_states.insert(
            *item_id,
            ItemState {
                collapsed: item_collapsed,
                dnd_idx: item_idx,
                dragging: item_dragging,
            },
        );
        *ui_state.dnd_idx += 1;

        if !item_collapsed && !item_dragging {
            draw_list(
                ghl,
                &list_id,
                &UiListConfig::SubList(UiSubListConfig { draw_header: false }),
                ui,
                ui_state,
            );
        }
    } else {
        ui.label(
            egui::RichText::new(format!("Could not find list id for item {item_id}"))
                .color(egui::Color32::RED),
        );
    }
}

fn find_start<ItemId, ListId>(
    root_list_id: &ListId,
    dnd_idx_start: usize,
    list_bounds: &HashMap<ListId, DndIdxBounds>,
    item_states: &HashMap<ItemId, ItemState>,
    ghl: &impl Ghl<ItemId = ItemId, ListId = ListId>,
) -> Option<DragLocation<ListId>>
where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    let mut curr_list_id = *root_list_id;
    'list_search: while let Some(bounds) = list_bounds.get(&curr_list_id) {
        if !bounds.contains(dnd_idx_start) {
            break 'list_search;
        }

        if let Some(children) = ghl.get_list_contents(&curr_list_id) {
            for (idx, item_id) in children.iter().enumerate() {
                if let (Some(item_state), Some(child_list_id)) =
                    (item_states.get(item_id), ghl.get_child_list_id(item_id))
                {
                    if item_state.dnd_idx == dnd_idx_start {
                        return Some(DragLocation {
                            list_id: curr_list_id,
                            idx,
                        });
                    }

                    if !item_state.collapsed
                        && list_bounds
                            .get(&child_list_id)
                            .map_or_else(|| false, |bounds| bounds.contains(dnd_idx_start))
                    {
                        curr_list_id = child_list_id;
                        continue 'list_search;
                    }
                } else {
                    break 'list_search;
                }
            }
        }

        break 'list_search;
    }

    None
}

fn find_end<ItemId, ListId>(
    root_list_id: &ListId,
    drag_start_list_id: &ListId,
    dnd_idx_end: usize,
    list_bounds: &HashMap<ListId, DndIdxBounds>,
    item_states: &HashMap<ItemId, ItemState>,
    ghl: &impl Ghl<ItemId = ItemId, ListId = ListId>,
) -> Option<DragDestination<ListId>>
where
    ItemId: Copy + Debug + Display + Eq + Hash,
    ListId: Copy + Debug + Display + Eq + Hash,
{
    let mut curr_list_id = *root_list_id;
    'list_search: while let Some(bounds) = list_bounds.get(&curr_list_id) {
        if let Some(children) = ghl.get_list_contents(&curr_list_id) {
            if bounds.end == dnd_idx_end {
                return Some(DragDestination::Push(curr_list_id));
            }

            for (idx, item_id) in children.iter().enumerate() {
                if let (Some(item_state), Some(child_list_id)) =
                    (item_states.get(item_id), ghl.get_child_list_id(item_id))
                {
                    if item_state.dnd_idx == dnd_idx_end {
                        if drag_start_list_id == &curr_list_id {
                            return Some(DragDestination::Within(idx));
                        }

                        return Some(DragDestination::Insert(DragLocation {
                            list_id: curr_list_id,
                            idx,
                        }));
                    }

                    if !item_state.collapsed
                        && !item_state.dragging
                        && list_bounds
                            .get(&child_list_id)
                            .map_or_else(|| false, |bounds| bounds.contains(dnd_idx_end))
                    {
                        curr_list_id = child_list_id;
                        continue 'list_search;
                    }
                } else {
                    break 'list_search;
                }
            }
        }

        break 'list_search;
    }

    None
}
