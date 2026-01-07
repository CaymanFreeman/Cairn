use crate::game::render::TextureType;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::HashMap;
use std::default::Default;

#[repr(u16)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub(crate) enum VoxelType {
    Air,
    Stone,
    Dirt,
    Grass,
}

pub(crate) struct VoxelProperties {
    textures: VoxelTextures,
    is_invisible: bool,
    is_occluding: bool,
}

impl Default for VoxelProperties {
    fn default() -> Self {
        Self {
            textures: VoxelTextures::uniform(TextureType::Stone),
            is_invisible: false,
            is_occluding: true,
        }
    }
}

impl VoxelProperties {
    pub(crate) fn is_occluding(&self) -> bool {
        self.is_occluding
    }

    pub(crate) fn is_invisible(&self) -> bool {
        self.is_invisible
    }

    pub(crate) fn front_texture(&self) -> TextureType {
        self.textures.front
    }

    pub(crate) fn back_texture(&self) -> TextureType {
        self.textures.back
    }

    pub(crate) fn right_texture(&self) -> TextureType {
        self.textures.right
    }

    pub(crate) fn left_texture(&self) -> TextureType {
        self.textures.left
    }

    pub(crate) fn top_texture(&self) -> TextureType {
        self.textures.top
    }

    pub(crate) fn bottom_texture(&self) -> TextureType {
        self.textures.bottom
    }
}

pub(crate) struct VoxelTextures {
    front: TextureType,
    back: TextureType,
    right: TextureType,
    left: TextureType,
    top: TextureType,
    bottom: TextureType,
}

impl VoxelTextures {
    pub(crate) fn uniform(texture: TextureType) -> Self {
        Self {
            front: texture,
            back: texture,
            right: texture,
            left: texture,
            top: texture,
            bottom: texture,
        }
    }

    pub(crate) fn top_bottom(
        top_texture: TextureType,
        bottom_texture: TextureType,
        side_texture: TextureType,
    ) -> Self {
        Self {
            front: side_texture,
            back: side_texture,
            right: side_texture,
            left: side_texture,
            top: top_texture,
            bottom: bottom_texture,
        }
    }
}

pub(crate) struct VoxelRegistry {
    properties: HashMap<VoxelType, VoxelProperties>,
}

impl VoxelRegistry {
    pub(crate) fn get_properties(&self, voxel_type: &VoxelType) -> &VoxelProperties {
        self.properties
            .get(voxel_type)
            .unwrap_or_else(|| panic!("Properties should exist for voxel type: {voxel_type:?}"))
    }

    pub(crate) fn init() -> Self {
        Self {
            properties: HashMap::from([
                (
                    VoxelType::Air,
                    VoxelProperties {
                        textures: VoxelTextures::uniform(TextureType::Air),
                        is_invisible: true,
                        is_occluding: false,
                    },
                ),
                (
                    VoxelType::Stone,
                    VoxelProperties {
                        textures: VoxelTextures::uniform(TextureType::Stone),
                        ..Default::default()
                    },
                ),
                (
                    VoxelType::Dirt,
                    VoxelProperties {
                        textures: VoxelTextures::uniform(TextureType::Dirt),
                        ..Default::default()
                    },
                ),
                (
                    VoxelType::Grass,
                    VoxelProperties {
                        textures: VoxelTextures::top_bottom(
                            TextureType::GrassTop,
                            TextureType::Dirt,
                            TextureType::GrassSide,
                        ),
                        ..Default::default()
                    },
                ),
            ]),
        }
    }
}
