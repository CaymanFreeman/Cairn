pub(crate) mod chunk;

mod mesh;
pub(crate) mod voxel;
pub(crate) use mesh::Mesh;
pub(crate) use mesh::Vertex;

pub(crate) use voxel::Voxel;

pub(crate) struct World {}

impl World {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
