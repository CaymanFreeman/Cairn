use crate::game::chunk::CHUNK_SIZE;

#[derive(Copy, Clone)]
pub(crate) struct WorldPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldPosition {
    pub(crate) fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn get_f32(&self) -> (f32, f32, f32) {
        (self.x as f32, self.y as f32, self.z as f32)
    }

    pub(crate) fn chunk_position(&self) -> ChunkPosition {
        let (x, y, z) = (
            self.x.div_euclid(CHUNK_SIZE as i32),
            self.y.div_euclid(CHUNK_SIZE as i32),
            self.z.div_euclid(CHUNK_SIZE as i32),
        );
        ChunkPosition::new(x, y, z)
    }

    pub(crate) fn local_chunk_position(&self) -> (ChunkPosition, LocalChunkPosition) {
        let chunk_position = self.chunk_position();
        let (chunk_x, chunk_y, chunk_z) = chunk_position.get();
        let (x, y, z) = (
            self.x - chunk_x * CHUNK_SIZE as i32,
            self.y - chunk_y * CHUNK_SIZE as i32,
            self.z - chunk_z * CHUNK_SIZE as i32,
        );
        (
            chunk_position,
            LocalChunkPosition::new(x as usize, y as usize, z as usize),
        )
    }

    pub(crate) fn front(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }

    pub(crate) fn back(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z - 1,
        }
    }

    pub(crate) fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub(crate) fn left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    pub(crate) fn top(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub(crate) fn bottom(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct ChunkPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl ChunkPosition {
    pub(crate) fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn get(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct LocalChunkPosition {
    x: usize,
    y: usize,
    z: usize,
}

impl LocalChunkPosition {
    pub(crate) fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn get(&self) -> (usize, usize, usize) {
        (self.x, self.y, self.z)
    }

    fn get_i32(&self) -> (i32, i32, i32) {
        (self.x as i32, self.y as i32, self.z as i32)
    }

    pub(crate) fn world_position(&self, chunk_position: ChunkPosition) -> WorldPosition {
        let (local_x, local_y, local_z) = self.get_i32();
        let (offset_x, offset_y, offset_z) = (
            chunk_position.x * CHUNK_SIZE as i32,
            chunk_position.y * CHUNK_SIZE as i32,
            chunk_position.z * CHUNK_SIZE as i32,
        );
        WorldPosition::new(local_x + offset_x, local_y + offset_y, local_z + offset_z)
    }
}
