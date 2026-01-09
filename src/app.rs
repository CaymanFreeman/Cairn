use crate::game::render::Renderer;
use crate::game::world::World;
use log::error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Icon, Window};

const WINDOW_ICON: &[u8] = include_bytes!("../assets/icon.png");

pub struct App {
    renderer: Option<Renderer>,
    world: Option<World>,
    mouse_captured: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            world: None,
            mouse_captured: false,
        }
    }

    fn update(&mut self) {
        if let (Some(renderer), Some(world)) = (&mut self.renderer, &mut self.world) {
            renderer.update();
            let chunk_position = renderer.camera().position().chunk_position();

            if world.last_update_position() != Some(chunk_position) {
                world.update_chunks(chunk_position);
                renderer.update_mesh(world);
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(renderer) = &mut self.renderer {
            renderer.render()?;
        }
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    fn grab_mouse(&mut self) {
        if let Some(renderer) = &self.renderer {
            let window = renderer.window();

            if !self.mouse_captured {
                window
                    .set_cursor_grab(CursorGrabMode::Confined)
                    .expect("Window should grab cursor");
                self.mouse_captured = true;
                window.set_cursor_visible(!self.mouse_captured);
            }
        }
    }

    fn release_mouse(&mut self) {
        if let Some(renderer) = &self.renderer {
            let window = renderer.window();

            if self.mouse_captured {
                window
                    .set_cursor_grab(CursorGrabMode::None)
                    .expect("Window should let go of cursor");
                self.mouse_captured = false;
                window.set_cursor_visible(!self.mouse_captured);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let mut world = World::new();

        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(WINDOW_ICON)
                .expect("Icon image should load")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect("Icon image should build from rgba");

        let window_attributes = Window::default_attributes()
            .with_title("Cairn")
            .with_window_icon(Some(icon))
            .with_maximized(true);

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Window should be created"),
        );

        let renderer = match pollster::block_on(Renderer::new(window, &world)) {
            Ok(mut renderer) => {
                let chunk_position = renderer.camera().position().chunk_position();
                world.update_chunks(chunk_position);
                renderer.update_mesh(&world);
                renderer
            }
            Err(error) => {
                error!("Failed to create renderer: {error}");
                event_loop.exit();
                return;
            }
        };

        self.world = Some(world);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        if let Some(renderer) = &self.renderer {
                            let size = renderer.window().inner_size();
                            self.resize(size.width, size.height);
                        }
                    }
                    Err(error) => {
                        error!("Unable to render: {error}");
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let is_pressed = key_state.is_pressed();
                if code == KeyCode::Escape && is_pressed {
                    self.release_mouse();
                }
                if let Some(renderer) = &mut self.renderer {
                    renderer
                        .camera_controller()
                        .handle_keyboard_input(code, is_pressed);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.grab_mouse();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if !self.mouse_captured {
            return;
        }

        let DeviceEvent::MouseMotion { delta } = event else {
            return;
        };

        let Some(renderer) = &mut self.renderer else {
            return;
        };

        renderer
            .camera_controller()
            .handle_mouse_input(delta.0 as f32, delta.1 as f32);
    }
}
