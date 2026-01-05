use crate::world::VoxelRegistry;
use image::{GenericImage as _, GenericImageView as _};
use log::{info, warn};

const TEXTURES_PATH: &str = "assets/textures/voxels";
const TEXTURE_FILE_EXTENSION: &str = "png";
const FALLBACK_TEXTURE_COLOR: image::Rgba<u8> = image::Rgba([255, 0, 255, 255]);

pub(crate) struct Texture {
    view: wgpu::TextureView,
    bind_group: Option<wgpu::BindGroup>,
}

impl Texture {
    pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn create_texture_atlas(registry: &VoxelRegistry) -> (image::DynamicImage, Vec<u32>) {
        let texture_order = registry.texture_order();
        let mut textures = Vec::new();
        let mut texture_widths = Vec::new();

        for texture_name in texture_order {
            let texture_path = format!("{TEXTURES_PATH}/{texture_name}.{TEXTURE_FILE_EXTENSION}");

            match image::open(&texture_path) {
                Ok(texture) => {
                    info!("Loading texture: {texture_path}");
                    texture_widths.push(texture.width());
                    textures.push(texture);
                }
                Err(error) => {
                    warn!("Failed to load texture {texture_path}: {error}");
                    let mut fallback = image::DynamicImage::new_rgba8(1, 1);
                    for pixel in fallback
                        .as_mut_rgba8()
                        .expect("Accessing fallback texture should not fail")
                        .pixels_mut()
                    {
                        *pixel = FALLBACK_TEXTURE_COLOR;
                    }
                    texture_widths.push(1);
                    textures.push(fallback);
                }
            }
        }

        if textures.is_empty() {
            warn!("No textures loaded, creating a 1x1 empty atlas");
            let empty = image::DynamicImage::new_rgba8(1, 1);
            return (empty, vec![1]);
        }

        let atlas_width: u32 = textures.iter().map(|texture| texture.width()).sum();
        let atlas_height: u32 = textures
            .iter()
            .map(|texture| texture.height())
            .max()
            .unwrap_or(1);

        info!("Stitching texture atlas ({atlas_width}x{atlas_height})...");
        let mut texture_atlas = image::DynamicImage::new_rgba8(atlas_width, atlas_height);
        let mut x_offset = 0;

        for texture in textures {
            texture_atlas
                .copy_from(&texture, x_offset, 0)
                .expect("Copying into atlas should never fail");
            x_offset += texture.width();
        }

        (texture_atlas, texture_widths)
    }

    pub(crate) fn create_diffuse_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        label: &str,
        registry: &VoxelRegistry,
    ) -> (Self, Vec<u32>) {
        let (texture_atlas, texture_widths) = Self::create_texture_atlas(registry);
        let rgba = texture_atlas.to_rgba8();
        let dimensions = texture_atlas.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        }));

        (Self { view, bind_group }, texture_widths)
    }

    pub(crate) fn view(&self) -> wgpu::TextureView {
        self.view.clone()
    }

    pub(crate) fn bind_group(&self) -> Option<wgpu::BindGroup> {
        self.bind_group.clone()
    }

    pub(crate) fn new_depth_texture(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            view,
            bind_group: None,
        }
    }
}
