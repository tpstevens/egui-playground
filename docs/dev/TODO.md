# MVP
- [ ] Make item title editable when clicking on it (to prioritize fast editing experience for simple items)
  - Try `egui::Response::highlight()` when hovered

# Next features

- [ ] Add way to bring up a modal dialogue with item description, additional notes, etc (middle click?)
  - [ ] Add decoration to item indicating that there are additional details to view

# Quality of life improvements

- [x] Add confirmation dialogue when removing an item with children
- [ ] Add function to reindex database
- [ ] Hide collapse and sort order from item when no children
- [ ] Add right-click context menu to item delete button that adds option to merge children with parent list
- [ ] Add info button that describes hidden context menu actions
- [ ] Add toggle for dark mode

# Under consideration

- [ ] Figure out how to handle nested clicking of different types (e.g. identify right-clicks on a frame that contains a button, which only checks for left clicks)
  - If not possible, consider implications for double-clicking list header to add item and right-clicking item for context menu
- [ ] Consider turning the "Sorted by" into a dropdown that allows showing different views
  - Different view possibilities:
    - List (sorting specified with user-provided string)
    - Node graph (for dependency order)
    - ?
  - By default, list view is a list that is sorted by "unsorted"
  - Auto-complete list order names (like ADO tags) to eventually allow for a command that applies a particular view to all lists that have it? e.g. order all lists that have a "priority" tag by priority.
    - After selecting "global" view, highlight items that don't have it?
  - Open up item dialogue to change a list's setting. Item dialogue could show the items in the list (without children)
- [ ] Make functions for converting IDs in the data model into egui IDs (e.g. `format!("list_{list_id}")` for list IDs