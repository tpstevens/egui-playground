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
