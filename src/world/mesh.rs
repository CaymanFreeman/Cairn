#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 3],
    texture_coordinates: [f32; 2],
}

impl Vertex {
    pub(crate) fn new(position: [f32; 3], texture_coordinates: [f32; 2]) -> Self {
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
    indices: Vec<u32>,
}

impl Mesh {
    pub(crate) fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub(crate) fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub(crate) fn vertices_u8(&self) -> &[u8] {
        bytemuck::cast_slice(self.vertices.as_slice())
    }

    pub(crate) fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub(crate) fn indices_u8(&self) -> &[u8] {
        bytemuck::cast_slice(self.indices.as_slice())
    }

    pub(crate) fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}
