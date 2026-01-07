use crate::game::mesh::OccludingVoxelNeighbors;
use crate::game::voxel::{VoxelRegistry, VoxelType};
use crate::game::world::{ChunkPosition, LocalChunkPosition};
use log::warn;
use std::ops::RangeInclusive;

pub(crate) const CHUNK_SIZE: usize = 32;

#[derive(Clone)]
pub(crate) struct Chunk {
    position: ChunkPosition,
    voxels: Vec<u16>,
}

impl Chunk {
    fn empty(position: ChunkPosition) -> Self {
        Self {
            position,
            voxels: vec![VoxelType::Air.into(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub(crate) fn dev_chunk(position: ChunkPosition) -> Self {
        let mut chunk = Self::empty(position);

        chunk.set_y_slice(31, VoxelType::Grass);
        chunk.set_y_range(27..=30, VoxelType::Dirt);
        chunk.set_y_range(0..=26, VoxelType::Stone);

        chunk
    }

    pub(crate) fn position(&self) -> ChunkPosition {
        self.position
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub(crate) fn get_occluding_neighbors(
        &self,
        local_position: LocalChunkPosition,
        voxel_registry: &VoxelRegistry,
    ) -> OccludingVoxelNeighbors {
        let front = {
            let front_neighbor = local_position.front();
            self.get_is_occluding(front_neighbor, voxel_registry)
        };
        let back = {
            let back_neighbor = local_position.back();
            self.get_is_occluding(back_neighbor, voxel_registry)
        };
        let right = {
            let right_neighbor = local_position.right();
            self.get_is_occluding(right_neighbor, voxel_registry)
        };
        let left = {
            let left_neighbor = local_position.left();
            self.get_is_occluding(left_neighbor, voxel_registry)
        };
        let top = {
            let top_neighbor = local_position.top();
            self.get_is_occluding(top_neighbor, voxel_registry)
        };
        let bottom = {
            let bottom_neighbor = local_position.bottom();
            self.get_is_occluding(bottom_neighbor, voxel_registry)
        };

        OccludingVoxelNeighbors::new(front, back, right, left, top, bottom)
    }

    #[expect(clippy::indexing_slicing)]
    fn set_voxel(&mut self, local_position: LocalChunkPosition, voxel_type: VoxelType) {
        let (x, y, z) = local_position.get();
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            warn!("Attempted to set voxel outside chunk bounds: ({x}, {y}, {z})");
            return;
        }

        self.voxels[Self::index(x, y, z)] = voxel_type.into();
    }

    fn set_y_slice(&mut self, y: usize, voxel_type: VoxelType) {
        if y >= CHUNK_SIZE {
            warn!("Attempted to set y-slice higher than chunk height: ({y})");
            return;
        }

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let local_position = LocalChunkPosition::new(x, y, z);
                self.set_voxel(local_position, voxel_type);
            }
        }
    }

    fn set_y_range(&mut self, y_range: RangeInclusive<usize>, voxel_type: VoxelType) {
        for y in y_range {
            self.set_y_slice(y, voxel_type);
        }
    }

    #[expect(clippy::indexing_slicing)]
    pub(crate) fn get_voxel_type(&self, local_position: LocalChunkPosition) -> VoxelType {
        let (x, y, z) = local_position.get();
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return VoxelType::Air;
        }

        VoxelType::try_from(self.voxels[Self::index(x, y, z)])
            .expect("Chunks should not store invalid voxel types")
    }

    fn get_is_occluding(
        &self,
        local_position: LocalChunkPosition,
        voxel_registry: &VoxelRegistry,
    ) -> bool {
        let voxel_type = self.get_voxel_type(local_position);
        voxel_registry.get_properties(&voxel_type).is_occluding()
    }
}
