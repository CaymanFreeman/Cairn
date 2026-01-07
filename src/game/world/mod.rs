mod position;

pub(crate) use position::*;

use crate::game::chunk::Chunk;
use crate::game::render::TextureAtlas;
use crate::game::voxel::VoxelRegistry;

pub(crate) struct World {
    voxel_registry: VoxelRegistry,
    texture_atlas: TextureAtlas,
    chunks: Vec<Chunk>,
}

impl World {
    pub(crate) fn new(chunks: Vec<Chunk>) -> Self {
        let voxel_registry = VoxelRegistry::init();
        let texture_atlas = TextureAtlas::init();
        Self {
            voxel_registry,
            texture_atlas,
            chunks,
        }
    }

    pub(crate) fn dev_world() -> Self {
        Self::new(vec![
            Chunk::dev_chunk(ChunkPosition::new(-1, 0, -1)),
            Chunk::dev_chunk(ChunkPosition::new(0, 0, -1)),
            Chunk::dev_chunk(ChunkPosition::new(1, 0, -1)),
            Chunk::dev_chunk(ChunkPosition::new(-1, 0, 0)),
            Chunk::dev_chunk(ChunkPosition::new(0, 0, 0)),
            Chunk::dev_chunk(ChunkPosition::new(1, 0, 0)),
            Chunk::dev_chunk(ChunkPosition::new(-1, 0, 1)),
            Chunk::dev_chunk(ChunkPosition::new(0, 0, 1)),
            Chunk::dev_chunk(ChunkPosition::new(1, 0, 1)),
        ])
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
}
