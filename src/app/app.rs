use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// Importiere deinen Button aus der Library
use crate::library::{
    ui_objects::{
        Button,
    }
};

use super::app_ui; // Importiert deinen "Designer"

// Hier sammeln wir alle UI-Elemente
pub struct App {
    _window: Window,
    event_loop: EventLoop<()>,
    buttons: Vec<Button>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Modulares Winit Fenster")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 400.0))
            .build(&event_loop)
            .expect("Fenster konnte nicht erstellt werden");

        Self { 
            _window: window, 
            event_loop, 
            buttons: Vec::new(),
            
        }
    }

    pub fn run(self) {
        let event_loop = self.event_loop;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}