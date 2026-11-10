#[allow(dead_code)]
pub struct Button {
    pub id: &'static str,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    // Eine Closure (Funktion), die bei einem Klick ausgeführt wird
    pub on_click: Box<dyn Fn()>,
}
