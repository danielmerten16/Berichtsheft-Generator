mod library;
mod app;

fn main() {
    let application = app::app::App::new();
    application.run();
}       