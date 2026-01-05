pub(crate) mod chunk;

mod mesh;
pub(crate) mod voxel;
pub(crate) use mesh::Mesh;
pub(crate) use mesh::Vertex;

pub(crate) use voxel::{Voxel, VoxelRegistry};

const VOXEL_DEFINITIONS_PATH: &str = "assets/data/definitions/voxels";

pub(crate) struct World {
    registry: VoxelRegistry,
}

impl World {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let registry = VoxelRegistry::load_from_directory(VOXEL_DEFINITIONS_PATH)?;

        Ok(Self { registry })
    }

    pub(crate) fn registry(&self) -> &VoxelRegistry {
        &self.registry
    }

    pub(crate) fn registry_mut(&mut self) -> &mut VoxelRegistry {
        &mut self.registry
    }
}
