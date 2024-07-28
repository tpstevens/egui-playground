#[derive(Hash)]
pub struct Item {
    pub id: usize,
    pub contents: &'static str,
}

impl Item {
    pub const fn new(id: usize) -> Self {
        Self {
            id,
            contents: "item description",
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui, handle: hello_egui::dnd::Handle) {
        ui.horizontal(|ui| {
            handle.ui(ui, |ui| {
                ui.label(self.id.to_string());
            });
            ui.label(self.contents);
        });
    }
}

pub fn move_elements_2<T>(
    list_1: &mut Vec<T>,
    list_2: &mut Vec<T>,
    from: usize,
    to: usize,
    num_elements_between: usize,
) {
    let num_sorted_items = list_1.len();

    if from < num_sorted_items {
        if to <= num_sorted_items {
            hello_egui::dnd::utils::shift_vec(from, to, list_1);
        } else {
            let removed_item = list_1.remove(from);
            list_2.insert(to - (num_sorted_items + num_elements_between), removed_item);
        }
    } else {
        let from_adj = from - (num_sorted_items + num_elements_between);
        if to >= num_sorted_items + num_elements_between {
            let to_adj = to - (num_sorted_items + num_elements_between);
            hello_egui::dnd::utils::shift_vec(from_adj, to_adj, list_2);
        } else {
            let removed_item = list_2.remove(from_adj);
            list_1.insert(to, removed_item);
        }
    }
}
