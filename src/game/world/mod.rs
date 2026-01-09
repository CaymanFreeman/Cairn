mod position;

use crate::game::chunk::Chunk;
use crate::game::mesh::OccludingVoxelNeighbors;
use crate::game::render::TextureAtlas;
use crate::game::voxel::{VoxelRegistry, VoxelType};
pub(crate) use position::*;
use std::collections::HashSet;

const RENDER_DISTANCE: u16 = 5;

pub(crate) struct World {
    voxel_registry: VoxelRegistry,
    texture_atlas: TextureAtlas,
    render_distance: u16,
    last_update_position: Option<ChunkPosition>,
    chunks: Vec<Chunk>,
}

impl World {
    pub(crate) fn new() -> Self {
        let voxel_registry = VoxelRegistry::init();
        let texture_atlas = TextureAtlas::init();
        Self {
            voxel_registry,
            texture_atlas,
            render_distance: RENDER_DISTANCE,
            last_update_position: None,
            chunks: vec![],
        }
    }

    pub(crate) fn update_chunks(&mut self, chunk_position: ChunkPosition) {
        self.last_update_position = Some(chunk_position);

        let render_distance = self.render_distance as i32;
        let (origin_x, origin_y, origin_z) = chunk_position.get();

        self.chunks.retain(|chunk| {
            let (chunk_x, chunk_y, chunk_z) = chunk.position().get();
            let distance_squared = (chunk_x - origin_x).pow(2)
                + (chunk_y - origin_y).pow(2)
                + (chunk_z - origin_z).pow(2);
            distance_squared <= render_distance.pow(2) && chunk_y == 0
        });

        let existing_chunks: HashSet<ChunkPosition> =
            self.chunks.iter().map(|chunk| chunk.position()).collect();

        let (min_x, max_x) = (origin_x - render_distance, origin_x + render_distance);
        let (min_y, max_y) = (origin_y - render_distance, origin_y + render_distance);
        let (min_z, max_z) = (origin_z - render_distance, origin_z + render_distance);

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let distance_squared =
                        (x - origin_x).pow(2) + (y - origin_y).pow(2) + (z - origin_z).pow(2);
                    if distance_squared <= render_distance.pow(2) && y == 0 {
                        let pos = ChunkPosition::new(x, y, z);
                        if !existing_chunks.contains(&pos) {
                            self.chunks.push(Chunk::dev_chunk(pos));
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn voxel_registry(&self) -> &VoxelRegistry {
        &self.voxel_registry
    }

    pub(crate) fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub(crate) fn chunks(&self) -> &Vec<Chunk> {
        &self.chunks
    }

    pub(crate) fn last_update_position(&self) -> Option<ChunkPosition> {
        self.last_update_position
    }

    fn get_chunk(&self, chunk_position: ChunkPosition) -> Option<&Chunk> {
        self.chunks
            .iter()
            .find(|chunk| chunk.position() == chunk_position)
    }

    pub(crate) fn get_voxel_type(&self, world_position: WorldPosition) -> VoxelType {
        let (chunk_position, local_chunk_position) = world_position.local_chunk_position();
        if let Some(chunk) = self.get_chunk(chunk_position) {
            chunk.get_voxel_type(local_chunk_position)
        } else {
            VoxelType::Air
        }
    }

    pub(crate) fn get_is_occluding(&self, world_position: WorldPosition) -> bool {
        let voxel_type = self.get_voxel_type(world_position);
        self.voxel_registry
            .get_properties(&voxel_type)
            .is_occluding()
    }

    pub(crate) fn get_occluding_neighbors(
        &self,
        world_position: WorldPosition,
    ) -> OccludingVoxelNeighbors {
        let front = {
            let front_neighbor = world_position.front();
            self.get_is_occluding(front_neighbor)
        };
        let back = {
            let back_neighbor = world_position.back();
            self.get_is_occluding(back_neighbor)
        };
        let right = {
            let right_neighbor = world_position.right();
            self.get_is_occluding(right_neighbor)
        };
        let left = {
            let left_neighbor = world_position.left();
            self.get_is_occluding(left_neighbor)
        };
        let top = {
            let top_neighbor = world_position.top();
            self.get_is_occluding(top_neighbor)
        };
        let bottom = {
            let bottom_neighbor = world_position.bottom();
            self.get_is_occluding(bottom_neighbor)
        };

        OccludingVoxelNeighbors::new(front, back, right, left, top, bottom)
    }
}
