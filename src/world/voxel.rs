use crate::world::{Mesh, Vertex};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum VoxelType {
    Solid,
}

pub(crate) struct Voxel {
    chunk_position: [u8; 3],
}

impl Voxel {
    pub(crate) fn new(position: [u8; 3]) -> Self {
        Self {
            chunk_position: position,
        }
    }
}

impl Voxel {
    pub(crate) fn generate_mesh(&self) -> Mesh {
        let (x, y, z) = (
            self.chunk_position[0] as f32,
            self.chunk_position[1] as f32,
            self.chunk_position[2] as f32,
        );

        #[rustfmt::skip]
        let vertices = vec![
            Vertex::new([x - 0.5, y - 0.5, z + 0.5], [0.0, 0.0]),
            Vertex::new([x + 0.5, y - 0.5, z + 0.5], [1.0, 0.0]),
            Vertex::new([x + 0.5, y + 0.5, z + 0.5], [1.0, 1.0]),
            Vertex::new([x - 0.5, y + 0.5, z + 0.5], [0.0, 1.0]),

            Vertex::new([x - 0.5, y + 0.5, z - 0.5], [1.0, 0.0]),
            Vertex::new([x + 0.5, y + 0.5, z - 0.5], [0.0, 0.0]),
            Vertex::new([x + 0.5, y - 0.5, z - 0.5], [0.0, 1.0]),
            Vertex::new([x - 0.5, y - 0.5, z - 0.5], [1.0, 1.0]),

            Vertex::new([x + 0.5, y - 0.5, z - 0.5], [0.0, 0.0]),
            Vertex::new([x + 0.5, y + 0.5, z - 0.5], [1.0, 0.0]),
            Vertex::new([x + 0.5, y + 0.5, z + 0.5], [1.0, 1.0]),
            Vertex::new([x + 0.5, y - 0.5, z + 0.5], [0.0, 1.0]),

            Vertex::new([x - 0.5, y - 0.5, z + 0.5], [1.0, 0.0]),
            Vertex::new([x - 0.5, y + 0.5, z + 0.5], [0.0, 0.0]),
            Vertex::new([x - 0.5, y + 0.5, z - 0.5], [0.0, 1.0]),
            Vertex::new([x - 0.5, y - 0.5, z - 0.5], [1.0, 1.0]),

            Vertex::new([x + 0.5, y + 0.5, z - 0.5], [1.0, 0.0]),
            Vertex::new([x - 0.5, y + 0.5, z - 0.5], [0.0, 0.0]),
            Vertex::new([x - 0.5, y + 0.5, z + 0.5], [0.0, 1.0]),
            Vertex::new([x + 0.5, y + 0.5, z + 0.5], [1.0, 1.0]),

            Vertex::new([x + 0.5, y - 0.5, z + 0.5], [0.0, 0.0]),
            Vertex::new([x - 0.5, y - 0.5, z + 0.5], [1.0, 0.0]),
            Vertex::new([x - 0.5, y - 0.5, z - 0.5], [1.0, 1.0]),
            Vertex::new([x + 0.5, y - 0.5, z - 0.5], [0.0, 1.0]),
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

        Mesh::new(vertices, indices)
    }
}
