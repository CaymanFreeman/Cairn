mod vertex;

pub(crate) use vertex::*;

use crate::game::chunk::{Chunk, CHUNK_SIZE};
use crate::game::render::TextureAtlas;
use crate::game::voxel::{VoxelProperties, VoxelRegistry};
use crate::game::world::{ChunkPosition, LocalChunkPosition, World, WorldPosition};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

#[derive(Clone)]
pub(crate) struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    fn merged(meshes: Vec<Self>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset: u32 = 0;

        for mesh in meshes {
            vertices.extend_from_slice(mesh.vertices());
            indices.extend(mesh.indices().iter().map(|i| i + vertex_offset));
            vertex_offset += mesh.vertices().len() as u32;
        }

        Self { vertices, indices }
    }

    pub(crate) fn voxel(
        world_position: WorldPosition,
        voxel_properties: &VoxelProperties,
        texture_atlas: &TextureAtlas,
        occluding_neighbors: &OccludingVoxelNeighbors,
    ) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let (x, y, z) = world_position.get_f32();

        if !occluding_neighbors.front {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.front_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_min, v_min]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }
        if !occluding_neighbors.back {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.back_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }
        if !occluding_neighbors.right {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.right_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_min, v_min]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_min, v_max]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }
        if !occluding_neighbors.left {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.left_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }
        if !occluding_neighbors.top {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.top_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }
        if !occluding_neighbors.bottom {
            let (u_min, u_max, v_min, v_max) = texture_atlas
                .get_coordinates(voxel_properties.bottom_texture())
                .get();
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_min, v_min]),
            ]);
            Self::extend_indices(&vertices, &mut indices);
        }

        Self { vertices, indices }
    }

    pub(crate) fn chunk(
        world: &World,
        chunk: &Chunk,
        voxel_registry: &VoxelRegistry,
        texture_atlas: &TextureAtlas,
    ) -> Self {
        let mut voxel_meshes = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_position = LocalChunkPosition::new(x, y, z);
                    let world_position = local_position.clone().world_position(chunk.position());
                    let voxel_type = chunk.get_voxel_type(local_position);
                    let voxel_properties = voxel_registry.get_properties(&voxel_type);
                    if voxel_properties.is_invisible() {
                        continue;
                    }
                    let occluding_neighbors = world.get_occluding_neighbors(world_position);
                    voxel_meshes.push(Self::voxel(
                        world_position,
                        voxel_properties,
                        texture_atlas,
                        &occluding_neighbors,
                    ));
                }
            }
        }

        Self::merged(voxel_meshes)
    }

    pub(crate) fn world(world: &mut World) -> Self {
        let voxel_registry = world.voxel_registry();
        let texture_atlas = world.texture_atlas();

        let chunk_positions = world
            .chunk_data()
            .keys()
            .copied()
            .collect::<Vec<ChunkPosition>>();

        let uncached_chunks = chunk_positions
            .iter()
            .filter(|chunk_position| world.chunk_meshes().get(chunk_position).is_none())
            .copied()
            .collect::<Vec<ChunkPosition>>();

        let new_chunk_meshes = uncached_chunks
            .into_par_iter()
            .map(|chunk_position| {
                let chunk = match world.chunk_data().get(&chunk_position) {
                    Some(chunk) => chunk,
                    None => &Chunk::empty(chunk_position),
                };
                let chunk_mesh = Self::chunk(world, chunk, voxel_registry, texture_atlas);
                (chunk_position, chunk_mesh)
            })
            .collect::<Vec<(ChunkPosition, Self)>>();

        for (chunk_position, chunk_mesh) in &new_chunk_meshes {
            world.insert_chunk_mesh(chunk_position, chunk_mesh.clone());
        }

        let all_chunk_meshes = chunk_positions
            .into_iter()
            .map(|chunk_position| {
                world
                    .chunk_meshes()
                    .get(&chunk_position)
                    .expect("Should have generated or cached each chunk mesh")
                    .clone()
            })
            .collect();

        Self::merged(all_chunk_meshes)
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

    fn extend_indices(vertices: &[Vertex], indices: &mut Vec<u32>) {
        let vertex_count = vertices.len() as u32;
        indices.extend(vec![
            vertex_count,
            vertex_count + 1,
            vertex_count + 2,
            vertex_count + 2,
            vertex_count + 3,
            vertex_count,
        ]);
    }
}

pub(crate) struct OccludingVoxelNeighbors {
    front: bool,
    back: bool,
    right: bool,
    left: bool,
    top: bool,
    bottom: bool,
}

impl OccludingVoxelNeighbors {
    pub(crate) fn new(
        front: bool,
        back: bool,
        right: bool,
        left: bool,
        top: bool,
        bottom: bool,
    ) -> Self {
        Self {
            front,
            back,
            right,
            left,
            top,
            bottom,
        }
    }
}
