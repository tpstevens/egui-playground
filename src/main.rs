mod dnd;
mod dnd_generic;

fn main() -> eframe::Result {
    // TODO: separate dnd and dnd_generic into crates with separate demo binaries
    dnd_generic::run_demo()
}
