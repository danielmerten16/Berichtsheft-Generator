use crate::library::ui_objects::{
    Button,
};

#[allow(dead_code)]
pub fn create_app_button1() -> Vec<Button> {
    let button1 = Button {
        id: "button1",
        label: "Button 1".into(),
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 50.0,
        on_click: Box::new(|| println!("Button 1 clicked!")),
    };

    vec![button1]
}
