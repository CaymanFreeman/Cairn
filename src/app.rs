use crate::renderer::Renderer;
use crate::world::World;
use log::error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct App {
    renderer: Option<Renderer>,
    world: World,
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
            world: World::new(),
        }
    }

    fn update(&self) {}

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
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title("Cairn");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());
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
            _ => {}
        }
    }
}
