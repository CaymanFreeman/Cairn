use crate::world::voxel::VoxelType;
use crate::world::{Mesh, Voxel};
use log::warn;
use std::ops::RangeInclusive;

const CHUNK_SIZE: usize = 32;
pub(crate) struct Chunk {
    position: [i32; 3],
    voxels: Vec<Option<VoxelType>>,
}

impl Chunk {
    pub(crate) fn new(position: [i32; 3]) -> Self {
        let mut chunk = Self {
            position,
            voxels: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        };

        chunk.set_y_range(0..=2, Some(VoxelType::Solid));

        chunk
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub(crate) fn generate_mesh(&self) -> Mesh {
        let mut voxel_meshes = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.get_voxel(x, y, z).is_some() {
                        let voxel = Voxel::new([x as u8, y as u8, z as u8]);
                        voxel_meshes.push(voxel.generate_mesh());
                    }
                }
            }
        }

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset: u32 = 0;

        for mesh in voxel_meshes {
            vertices.extend_from_slice(mesh.vertices());
            indices.extend(mesh.indices().iter().map(|i| i + vertex_offset));
            vertex_offset += mesh.vertices().len() as u32;
        }

        Mesh::new(vertices, indices)
    }

    #[expect(clippy::indexing_slicing)]
    pub(crate) fn set_voxel(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        voxel_type: Option<VoxelType>,
    ) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            warn!("Attempted to set voxel outside chunk bounds: ({x}, {y}, {z})");
            return;
        }

        self.voxels[Self::index(x, y, z)] = voxel_type;
    }

    pub(crate) fn set_y_slice(&mut self, y: usize, voxel_type: Option<VoxelType>) {
        if y >= CHUNK_SIZE {
            warn!("Attempted to set y-slice higher than chunk height: ({y})");
            return;
        }

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                self.set_voxel(x, y, z, voxel_type);
            }
        }
    }

    pub(crate) fn set_y_range(
        &mut self,
        y_range: RangeInclusive<usize>,
        voxel_type: Option<VoxelType>,
    ) {
        for y in y_range {
            self.set_y_slice(y, voxel_type);
        }
    }

    #[expect(clippy::indexing_slicing)]
    pub(crate) fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<VoxelType> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            warn!("Attempted to get voxel outside chunk bounds: ({x}, {y}, {z})");
            return None;
        }

        self.voxels[Self::index(x, y, z)]
    }
}
