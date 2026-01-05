use crate::world::{voxel, Mesh, Voxel, VoxelRegistry};
use log::warn;
use std::ops::RangeInclusive;

const CHUNK_SIZE: usize = 32;

pub(crate) struct Chunk {
    position: [i32; 3],
    voxels: Vec<Option<u16>>,
}

impl Chunk {
    fn empty(position: [i32; 3]) -> Self {
        Self {
            position,
            voxels: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub(crate) fn new(position: [i32; 3], registry: &VoxelRegistry) -> Self {
        let mut chunk = Self::empty(position);

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

    fn get_voxel_exposed_faces(&self, chunk_position: [u8; 3]) -> Vec<voxel::Face> {
        let mut faces = Vec::new();
        let (x, y, z) = (
            chunk_position[0] as isize,
            chunk_position[1] as isize,
            chunk_position[2] as isize,
        );

        if self.get_voxel(x, y, z + 1).is_none() {
            faces.push(voxel::Face::Front);
        }
        if self.get_voxel(x, y, z - 1).is_none() {
            faces.push(voxel::Face::Back);
        }
        if self.get_voxel(x + 1, y, z).is_none() {
            faces.push(voxel::Face::Right);
        }
        if self.get_voxel(x - 1, y, z).is_none() {
            faces.push(voxel::Face::Left);
        }
        if self.get_voxel(x, y + 1, z).is_none() {
            faces.push(voxel::Face::Top);
        }
        if self.get_voxel(x, y - 1, z).is_none() {
            faces.push(voxel::Face::Bottom);
        }
        faces
    }

    pub(crate) fn generate_mesh(&self, registry: &VoxelRegistry) -> Mesh {
        let mut voxel_meshes = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if let Some(definition_id) = self.get_voxel(x as isize, y as isize, z as isize)
                    {
                        let chunk_position = [x as u8, y as u8, z as u8];
                        let voxel = Voxel::new(chunk_position, definition_id);
                        voxel_meshes.push(voxel.generate_mesh(
                            registry,
                            &self.get_voxel_exposed_faces(chunk_position),
                        ));
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
    pub(crate) fn get_voxel(&self, x: isize, y: isize, z: isize) -> Option<u16> {
        if x < 0
            || y < 0
            || z < 0
            || x >= CHUNK_SIZE as isize
            || y >= CHUNK_SIZE as isize
            || z >= CHUNK_SIZE as isize
        {
            return None;
        }

        self.voxels[Self::index(x as usize, y as usize, z as usize)]
    }
}
