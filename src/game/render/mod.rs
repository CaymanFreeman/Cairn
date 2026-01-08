mod atlas;
mod camera;
mod texture;

pub(crate) use atlas::*;
pub(crate) use camera::*;
pub(crate) use texture::*;

use crate::game::mesh::{Mesh, Vertex};
use crate::game::render::Texture;
use crate::game::render::{Camera, CameraController};
use crate::game::world::World;
use std::sync::Arc;
use wgpu::util::DeviceExt as _;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const WORLD_SHADER: wgpu::ShaderModuleDescriptor<'_> =
    wgpu::include_wgsl!("../../../shaders/voxel.wgsl");

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
    pub(crate) async fn new(window: Arc<Window>, world: &World) -> anyhow::Result<Self> {
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

        let texture_atlas_image = &world.texture_atlas().image();

        let texture_bind_group_layout = create_texture_bind_group_layout(&device);
        let diffuse_texture = Texture::create_diffuse_texture(
            &device,
            &queue,
            &texture_bind_group_layout,
            texture_atlas_image,
            "texture_atlas",
        );

        let depth_texture = Texture::new_depth_texture(&device, &surface_config, "Depth Texture");

        let camera_bind_group_layout = create_camera_bind_group_layout(&device);
        let camera = Camera::new(&device, &surface_config, &camera_bind_group_layout);
        let camera_controller = CameraController::new();

        let world_mesh = Mesh::world(world);
        let vertex_buffer = create_vertex_buffer(&device, world_mesh.vertices_u8());
        let index_buffer = create_index_buffer(&device, world_mesh.indices_u8());
        let index_count = world_mesh.index_count();

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

    pub(crate) fn camera(&self) -> &Camera {
        &self.camera
    }

    pub(crate) fn camera_controller(&mut self) -> &mut CameraController {
        &mut self.camera_controller
    }

    pub(crate) fn update_mesh(&mut self, world: &World) {
        let world_mesh = Mesh::world(world);
        self.vertex_buffer = create_vertex_buffer(&self.device, world_mesh.vertices_u8());
        self.index_buffer = create_index_buffer(&self.device, world_mesh.indices_u8());
        self.index_count = world_mesh.index_count();
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

        if self.index_count > 0 {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_texture.bind_group(), &[]);
            render_pass.set_bind_group(1, &self.camera.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
fn create_surface_config(
    window_size: PhysicalSize<u32>,
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|texture_format| texture_format.is_srgb())
        .copied()
        .unwrap_or(
            *surface_caps
                .formats
                .first()
                .expect("Should access preferred texture format"),
        );

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(WORLD_SHADER);
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview_mask: None,
    })
}

fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })
}

fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    })
}

fn create_vertex_buffer(device: &wgpu::Device, contents: &[u8]) -> wgpu::Buffer {
    if contents.is_empty() {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer (empty)"),
            contents: &[0u8; 1],
            usage: wgpu::BufferUsages::VERTEX,
        });
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents,
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_index_buffer(device: &wgpu::Device, contents: &[u8]) -> wgpu::Buffer {
    if contents.is_empty() {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer (empty)"),
            contents: &[0u8; 1],
            usage: wgpu::BufferUsages::INDEX,
        });
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents,
        usage: wgpu::BufferUsages::INDEX,
    })
}
