use crate::world::{Mesh, Voxel, VoxelRegistry};
use log::warn;
use std::ops::RangeInclusive;

const CHUNK_SIZE: usize = 32;

pub(crate) struct Chunk {
    _position: [i32; 3],
    voxels: Vec<Option<u16>>,
}

impl Chunk {
    pub(crate) fn new(_position: [i32; 3], registry: &VoxelRegistry) -> Self {
        let mut chunk = Self {
            _position,
            voxels: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        };

        if let Some(id) = registry.get_id("stone") {
            chunk.set_y_range(0..=5, Some(id));
        }

        if let Some(id) = registry.get_id("dirt") {
            chunk.set_y_range(6..=7, Some(id));
        }

        if let Some(id) = registry.get_id("grass") {
            chunk.set_y_range(8..=8, Some(id));
        }

        chunk
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub(crate) fn generate_mesh(&self, registry: &VoxelRegistry) -> Mesh {
        let mut voxel_meshes = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if let Some(definition_id) = self.get_voxel(x, y, z) {
                        let voxel = Voxel::new([x as u8, y as u8, z as u8], definition_id);
                        voxel_meshes.push(voxel.generate_mesh(registry));
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
    pub(crate) fn set_voxel(&mut self, x: usize, y: usize, z: usize, definition_id: Option<u16>) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            warn!("Attempted to set voxel outside chunk bounds: ({x}, {y}, {z})");
            return;
        }

        self.voxels[Self::index(x, y, z)] = definition_id;
    }

    pub(crate) fn set_voxel_by_type(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        voxel_type: Option<&str>,
        registry: &VoxelRegistry,
    ) {
        let definition_id = voxel_type.and_then(|voxel_type| registry.get_id(voxel_type));
        self.set_voxel(x, y, z, definition_id);
    }

    pub(crate) fn set_y_slice(&mut self, y: usize, definition_id: Option<u16>) {
        if y >= CHUNK_SIZE {
            warn!("Attempted to set y-slice higher than chunk height: ({y})");
            return;
        }

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                self.set_voxel(x, y, z, definition_id);
            }
        }
    }

    pub(crate) fn set_y_range(
        &mut self,
        y_range: RangeInclusive<usize>,
        definition_id: Option<u16>,
    ) {
        for y in y_range {
            self.set_y_slice(y, definition_id);
        }
    }

    #[expect(clippy::indexing_slicing)]
    pub(crate) fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<u16> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            warn!("Attempted to get voxel outside chunk bounds: ({x}, {y}, {z})");
            return None;
        }

        self.voxels[Self::index(x, y, z)]
    }
}
