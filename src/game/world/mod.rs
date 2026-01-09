use rayon::iter::ParallelIterator;
mod position;

pub(crate) use position::*;

use crate::game::chunk::Chunk;
use crate::game::mesh::{Mesh, OccludingVoxelNeighbors};
use crate::game::render::TextureAtlas;
use crate::game::voxel::{VoxelRegistry, VoxelType};
use rayon::iter::IntoParallelRefIterator as _;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

const RENDER_DISTANCE_XZ: i32 = 6;
const RENDER_DISTANCE_Y: i32 = 3;
const CHUNK_RENDER_MAXIMUM: usize =
    (PI * RENDER_DISTANCE_XZ.pow(2) as f32 * (2 * RENDER_DISTANCE_Y + 1) as f32).ceil() as usize;

pub(crate) struct World {
    voxel_registry: VoxelRegistry,
    texture_atlas: TextureAtlas,
    last_update_position: Option<ChunkPosition>,
    chunk_data: HashMap<ChunkPosition, Chunk>,
    chunk_meshes: HashMap<ChunkPosition, Mesh>,
}

impl World {
    pub(crate) fn new() -> Self {
        let voxel_registry = VoxelRegistry::init();
        let texture_atlas = TextureAtlas::init();
        Self {
            voxel_registry,
            texture_atlas,
            last_update_position: None,
            chunk_data: HashMap::new(),
            chunk_meshes: HashMap::new(),
        }
    }

    pub(crate) fn update_chunks(&mut self, origin_chunk_position: ChunkPosition) {
        self.last_update_position = Some(origin_chunk_position);

        let chunks_in_range_vec = Self::determine_chunks_in_range(origin_chunk_position);
        let chunks_in_range_set = chunks_in_range_vec
            .par_iter()
            .copied()
            .collect::<HashSet<ChunkPosition>>();
        self.unload_out_of_range_chunks(&chunks_in_range_set);
        self.load_in_range_chunks(&chunks_in_range_vec);
    }

    fn determine_chunks_in_range(origin_chunk_position: ChunkPosition) -> Vec<ChunkPosition> {
        let (origin_x, origin_y, origin_z) = origin_chunk_position.get();
        let (min_x, max_x) = (origin_x - RENDER_DISTANCE_XZ, origin_x + RENDER_DISTANCE_XZ);
        let (min_y, max_y) = (origin_y - RENDER_DISTANCE_Y, origin_y + RENDER_DISTANCE_Y);
        let (min_z, max_z) = (origin_z - RENDER_DISTANCE_XZ, origin_z + RENDER_DISTANCE_XZ);

        let mut chunks_in_range = Vec::with_capacity(CHUNK_RENDER_MAXIMUM);
        for x in min_x..=max_x {
            let distance_x_squared = (x - origin_x).pow(2);
            if distance_x_squared > RENDER_DISTANCE_XZ.pow(2) {
                continue;
            }

            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let distance_z_squared = (z - origin_z).pow(2);
                    let distance_squared = distance_x_squared + distance_z_squared;
                    if distance_squared <= RENDER_DISTANCE_XZ.pow(2) {
                        chunks_in_range.push(ChunkPosition::new(x, y, z));
                    }
                }
            }
        }

        chunks_in_range.sort_by_cached_key(|chunk_position| {
            let (x, y, z) = chunk_position.get();
            (x - origin_x).pow(2) + (y - origin_y).pow(2) + (z - origin_z).pow(2)
        });

        chunks_in_range
    }

    fn load_in_range_chunks(&mut self, chunks_in_range: &[ChunkPosition]) {
        for chunk_position in chunks_in_range {
            if !self.chunk_data.contains_key(chunk_position) {
                self.chunk_data
                    .insert(*chunk_position, Chunk::dev_chunk(*chunk_position));
            }
        }
    }

    fn unload_out_of_range_chunks(&mut self, chunks_in_range: &HashSet<ChunkPosition>) {
        self.chunk_data
            .retain(|pos, _chunk| chunks_in_range.contains(pos));
    }

    pub(crate) fn voxel_registry(&self) -> &VoxelRegistry {
        &self.voxel_registry
    }

    pub(crate) fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub(crate) fn chunk_data(&self) -> &HashMap<ChunkPosition, Chunk> {
        &self.chunk_data
    }

    pub(crate) fn chunk_meshes(&self) -> &HashMap<ChunkPosition, Mesh> {
        &self.chunk_meshes
    }

    pub(crate) fn insert_chunk_mesh(&mut self, chunk_position: &ChunkPosition, chunk_mesh: Mesh) {
        self.chunk_meshes.insert(*chunk_position, chunk_mesh);
    }

    pub(crate) fn last_update_position(&self) -> Option<ChunkPosition> {
        self.last_update_position
    }

    pub(crate) fn get_voxel_type(&self, world_position: WorldPosition) -> VoxelType {
        let (chunk_position, local_chunk_position) = world_position.local_chunk_position();
        match self.chunk_data.get(&chunk_position) {
            Some(chunk) => chunk.get_voxel_type(local_chunk_position),
            None => VoxelType::Air,
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
