use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

// =================================================================================================
// Public types, traits, and functions

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

/// State that must be passed into closures executed by `DndItemCache` functions
pub struct DndState<'a, 'b> {
    idx: &'a mut usize,
    item_iter: &'a mut hello_egui::dnd::item_iterator::ItemIterator<'b>,
}

/// Implementing this trait for a nested collection of lists of items makes it possible to render
/// them in a drag-and-drop `hello_egui::dnd` container. `hello_egui::dnd` currently does not
/// support nested lists out of the box, so drawing is split into a few methods to work around this
/// restriction.
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
/// It is similarly reasonable for the closure that draws the footer to do nothing, if desired.
// TODO: summary here, details on each function
pub trait DndItemCache {
    type ItemId: Copy + Eq + Display + Hash;
    type ListId: Copy + Eq + Display + Hash;

    // Get the list that belongs to the given item.
    fn get_child_list_id(&self, item_id: &Self::ItemId) -> Option<Self::ListId>;

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
        dnd_state: &mut DndState,
        ui_items: impl FnOnce(&mut Self, &mut egui::Ui, &mut DndState, &Vec<Self::ItemId>),
        ui_footer: impl FnOnce(&mut egui::Ui, &mut DndState, Box<dyn FnOnce(&mut egui::Ui)>),
    );
}

pub fn ui<ItemId, ListId>(
    ui: &mut egui::Ui,
    ui_id: egui::Id,
    item_cache: &mut impl DndItemCache<ItemId = ItemId, ListId = ListId>,
    root_list_id: &ListId,
) where
    ItemId: Copy + Eq + Display + Hash,
    ListId: Copy + Eq + Display + Hash,
{
    let mut dnd_idx = 0usize;
    let mut list_bounds = HashMap::<ListId, DndIdxBounds>::new();

    item_cache.ui_list_header(root_list_id, &UiListHeaderConfig::Root, ui);
    hello_egui::dnd::dnd(ui, ui_id).show_custom(|ui, item_iter| {
        let mut dnd_state = DndState {
            idx: &mut dnd_idx,
            item_iter,
        };

        draw_list(
            &mut list_bounds,
            item_cache,
            root_list_id,
            &UiListConfig::Root,
            ui,
            &mut dnd_state,
        );
    });
}

// =================================================================================================
// Private types, traits, and functions

struct DndIdxBounds {
    start: usize,
    end: usize,
}

impl DndIdxBounds {
    const fn contains(&self, dnd_idx: usize) -> bool {
        dnd_idx >= self.start && dnd_idx <= self.end
    }
}

fn draw_list<ItemId, ListId, T: DndItemCache<ItemId = ItemId, ListId = ListId>>(
    list_bounds: &mut HashMap<ListId, DndIdxBounds>,
    item_cache: &mut T,
    list_id: &ListId,
    config: &UiListConfig,
    ui: &mut egui::Ui,
    dnd_state: &mut DndState,
) where
    ItemId: Copy + Eq + Display + Hash,
    ListId: Copy + Eq + Display + Hash,
{
    let start = *dnd_state.idx;
    let mut end = start;

    let ui_items =
        |item_cache: &mut T, ui: &mut egui::Ui, dnd_state: &mut DndState, items: &Vec<ItemId>| {
            for item in items {
                draw_item(list_bounds, item_cache, item, ui, dnd_state);
            }
        };

    let ui_footer =
        |ui: &mut egui::Ui, dnd_state: &mut DndState, footer: Box<dyn FnOnce(&mut egui::Ui)>| {
            match config {
                UiListConfig::Root => {
                    // Don't include the footer in the root list to prevent items from being
                    // dragged past it
                    footer(ui);
                }
                UiListConfig::SubList(_) => {
                    dnd_state.item_iter.next(
                        ui,
                        egui::Id::new(list_id).with("footer"),
                        *dnd_state.idx,
                        true,
                        |ui, dnd_item| {
                            dnd_item.ui(ui, |ui, _handle, _item_state| {
                                footer(ui);
                            })
                        },
                    );
                    end = *dnd_state.idx;
                    *dnd_state.idx += 1;
                }
            }
        };

    item_cache.ui_list_contents(list_id, config, ui, dnd_state, ui_items, ui_footer);
    list_bounds.insert(*list_id, DndIdxBounds { start, end });
}

fn draw_item<ItemId, ListId, T: DndItemCache<ItemId = ItemId, ListId = ListId>>(
    list_bounds: &mut HashMap<ListId, DndIdxBounds>,
    item_cache: &mut T,
    item_id: &ItemId,
    ui: &mut egui::Ui,
    dnd_state: &mut DndState,
) where
    ItemId: Copy + Eq + Display + Hash,
    ListId: Copy + Eq + Display + Hash,
{
    let mut force_collapsed = false;
    match item_cache.get_child_list_id(item_id) {
        Some(list_id) => {
            dnd_state.item_iter.next(
                ui,
                egui::Id::new(item_id),
                *dnd_state.idx,
                true,
                |ui, dnd_item| {
                    dnd_item.ui(ui, |ui, handle, item_state| {
                        force_collapsed = item_state.dragged;

                        force_collapsed |= item_cache.ui_item(item_id, ui, handle, force_collapsed);

                        if !force_collapsed {
                            item_cache.ui_list_header(&list_id, &UiListHeaderConfig::SubList, ui);
                        }
                    })
                },
            );

            if !force_collapsed {
                draw_list(
                    list_bounds,
                    item_cache,
                    &list_id,
                    &UiListConfig::SubList(UiSubListConfig { draw_header: false }),
                    ui,
                    dnd_state,
                );
            }
        }
        None => {
            ui.label(format!("Could not find list id for item {item_id}"));
        }
    }
}
