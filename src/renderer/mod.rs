mod camera;
mod pipeline;
mod texture;

use crate::renderer::pipeline::create_camera_bind_group_layout;
use crate::world::chunk::Chunk;
use crate::world::World;
use camera::{Camera, CameraController};
use pipeline::{
    create_index_buffer, create_render_pipeline, create_surface_config,
    create_texture_bind_group_layout, create_vertex_buffer,
};
use std::sync::Arc;
use texture::Texture;
use winit::window::Window;

pub(crate) struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    diffuse_texture: Texture,
    depth_texture: Texture,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    camera: Camera,
    camera_controller: CameraController,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub(crate) async fn new(window: Arc<Window>, world: &mut World) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_config = create_surface_config(window.inner_size(), &surface, &adapter);

        let texture_bind_group_layout = create_texture_bind_group_layout(&device);

        let (diffuse_texture, texture_widths) = Texture::create_diffuse_texture(
            &device,
            &queue,
            &texture_bind_group_layout,
            "atlas.png",
            world.registry(),
        );

        let atlas_width: u32 = texture_widths.iter().sum();
        world
            .registry_mut()
            .update_texture_coordinates(&texture_widths, atlas_width);

        let depth_texture = Texture::new_depth_texture(&device, &surface_config, "Depth Texture");

        let camera_bind_group_layout = create_camera_bind_group_layout(&device);
        let camera = Camera::new(&device, &surface_config, &camera_bind_group_layout);
        let camera_controller = CameraController::new();

        let chunk_mesh = Chunk::new([0, 0, 0], world.registry()).generate_mesh(world.registry());
        let vertex_buffer = create_vertex_buffer(&device, chunk_mesh.vertices_u8());
        let index_buffer = create_index_buffer(&device, chunk_mesh.indices_u8());
        let index_count = chunk_mesh.index_count();

        let render_pipeline = create_render_pipeline(
            &device,
            &surface_config,
            &[&texture_bind_group_layout, &camera_bind_group_layout],
        );

        Ok(Self {
            window,
            surface,
            device,
            queue,
            surface_config,
            diffuse_texture,
            depth_texture,
            vertex_buffer,
            index_buffer,
            index_count,
            camera,
            camera_controller,
            render_pipeline,
        })
    }

    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    pub(crate) fn camera_controller(&mut self) -> &mut CameraController {
        &mut self.camera_controller
    }

    pub(crate) fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera.update_buffer(&self.queue);
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture =
                Texture::new_depth_texture(&self.device, &self.surface_config, "Depth Texture");
            self.camera.resize(width, height);
        }
    }

    pub(crate) fn render(&self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
