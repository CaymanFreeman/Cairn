#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 3],
    texture_coordinates: [f32; 2],
}

impl Vertex {
    fn new(position: [f32; 3], texture_coordinates: [f32; 2]) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }

    pub(crate) fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub(crate) struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Mesh {
    pub(crate) fn vertices(&self) -> &[u8] {
        bytemuck::cast_slice(self.vertices.as_slice())
    }

    pub(crate) fn indices(&self) -> &[u8] {
        bytemuck::cast_slice(self.indices.as_slice())
    }

    pub(crate) fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}

pub(crate) struct Voxel {
    position: [i32; 3],
}

impl Voxel {
    pub(crate) fn new(position: [i32; 3]) -> Self {
        Self { position }
    }
}

impl Voxel {
    pub(crate) fn generate_mesh(&self) -> Mesh {
        let (x, y, z) = (
            self.position[0] as f32,
            self.position[1] as f32,
            self.position[2] as f32,
        );

        #[rustfmt::skip]
        let vertices = vec![
            Vertex::new([x - 1.0, y - 1.0, z + 1.0], [0.0, 0.0]),
            Vertex::new([x + 1.0, y - 1.0, z + 1.0], [1.0, 0.0]),
            Vertex::new([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),
            Vertex::new([x - 1.0, y + 1.0, z + 1.0], [0.0, 1.0]),

            Vertex::new([x - 1.0, y + 1.0, z - 1.0], [1.0, 0.0]),
            Vertex::new([x + 1.0, y + 1.0, z - 1.0], [0.0, 0.0]),
            Vertex::new([x + 1.0, y - 1.0, z - 1.0], [0.0, 1.0]),
            Vertex::new([x - 1.0, y - 1.0, z - 1.0], [1.0, 1.0]),

            Vertex::new([x + 1.0, y - 1.0, z - 1.0], [0.0, 0.0]),
            Vertex::new([x + 1.0, y + 1.0, z - 1.0], [1.0, 0.0]),
            Vertex::new([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),
            Vertex::new([x + 1.0, y - 1.0, z + 1.0], [0.0, 1.0]),

            Vertex::new([x - 1.0, y - 1.0, z + 1.0], [1.0, 0.0]),
            Vertex::new([x - 1.0, y + 1.0, z + 1.0], [0.0, 0.0]),
            Vertex::new([x - 1.0, y + 1.0, z - 1.0], [0.0, 1.0]),
            Vertex::new([x - 1.0, y - 1.0, z - 1.0], [1.0, 1.0]),

            Vertex::new([x + 1.0, y + 1.0, z - 1.0], [1.0, 0.0]),
            Vertex::new([x - 1.0, y + 1.0, z - 1.0], [0.0, 0.0]),
            Vertex::new([x - 1.0, y + 1.0, z + 1.0], [0.0, 1.0]),
            Vertex::new([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),

            Vertex::new([x + 1.0, y - 1.0, z + 1.0], [0.0, 0.0]),
            Vertex::new([x - 1.0, y - 1.0, z + 1.0], [1.0, 0.0]),
            Vertex::new([x - 1.0, y - 1.0, z - 1.0], [1.0, 1.0]),
            Vertex::new([x + 1.0, y - 1.0, z - 1.0], [0.0, 1.0]),
        ];

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2, 2, 3, 0,
            4, 5, 6, 6, 7, 4,
            8, 9, 10, 10, 11, 8,
            12, 13, 14, 14, 15, 12,
            16, 17, 18, 18, 19, 16,
            20, 21, 22, 22, 23, 20,
        ];

        Mesh { vertices, indices }
    }
}
