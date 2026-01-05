use crate::world::{Mesh, Vertex};
use log::{error, info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(PartialEq)]
pub(crate) enum Face {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

#[derive(Copy, Clone)]
pub(crate) struct Voxel {
    chunk_position: [u8; 3],
    definition_id: u16,
}

impl Voxel {
    pub(crate) fn new(chunk_position: [u8; 3], definition_id: u16) -> Self {
        Self {
            chunk_position,
            definition_id,
        }
    }

    pub(crate) fn from_type(
        chunk_position: [u8; 3],
        definition_type: &str,
        registry: &VoxelRegistry,
    ) -> Option<Self> {
        registry
            .type_to_id
            .get(definition_type)
            .map(|&definition_id| Self::new(chunk_position, definition_id))
    }

    pub(crate) fn definition_id(&self) -> u16 {
        self.definition_id
    }

    pub(crate) fn generate_mesh(&self, registry: &VoxelRegistry, faces: &[Face]) -> Mesh {
        let (x, y, z) = (
            self.chunk_position[0] as f32,
            self.chunk_position[1] as f32,
            self.chunk_position[2] as f32,
        );

        let definition = registry
            .id_to_definition
            .get(&self.definition_id)
            .expect("Voxel definition should exist for valid definition_id");

        let (u_min, v_min, u_max, v_max) = definition.texture_coordinates;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if faces.contains(&Face::Front) {
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_min, v_min]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }
        if faces.contains(&Face::Back) {
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }
        if faces.contains(&Face::Right) {
            vertices.extend(vec![
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_min, v_min]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_min, v_max]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }
        if faces.contains(&Face::Left) {
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }
        if faces.contains(&Face::Top) {
            vertices.extend(vec![
                Vertex::new([x - 0.5, y + 0.5, z - 0.5], [u_min, v_min]),
                Vertex::new([x - 0.5, y + 0.5, z + 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z + 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y + 0.5, z - 0.5], [u_max, v_min]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }
        if faces.contains(&Face::Bottom) {
            vertices.extend(vec![
                Vertex::new([x - 0.5, y - 0.5, z - 0.5], [u_min, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z - 0.5], [u_max, v_max]),
                Vertex::new([x + 0.5, y - 0.5, z + 0.5], [u_max, v_min]),
                Vertex::new([x - 0.5, y - 0.5, z + 0.5], [u_min, v_min]),
            ]);
            Mesh::extend_indices(&vertices, &mut indices);
        }

        Mesh::new(vertices, indices)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct VoxelDefinitionJson {
    #[serde(rename = "type")]
    voxel_type: String,
    texture: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VoxelDefinition {
    pub(crate) voxel_type: String,
    pub(crate) texture_name: String,
    pub(crate) texture_coordinates: (f32, f32, f32, f32),
}

impl VoxelDefinition {
    fn from_json(json: VoxelDefinitionJson, texture_coordinates: (f32, f32, f32, f32)) -> Self {
        Self {
            voxel_type: json.voxel_type,
            texture_name: json.texture,
            texture_coordinates,
        }
    }
}

pub(crate) struct VoxelRegistry {
    type_to_id: HashMap<String, u16>,
    id_to_definition: HashMap<u16, VoxelDefinition>,
    texture_order: Vec<String>,
}

impl VoxelRegistry {
    pub(crate) fn new() -> Self {
        Self {
            type_to_id: HashMap::new(),
            id_to_definition: HashMap::new(),
            texture_order: Vec::new(),
        }
    }

    pub(crate) fn load_from_directory<P: AsRef<Path>>(definitions_path: P) -> anyhow::Result<Self> {
        let mut registry = Self::new();
        let definitions_path = definitions_path.as_ref();

        info!(
            "Loading voxel definitions from: {}",
            definitions_path.display()
        );

        let entries = std::fs::read_dir(definitions_path).map_err(|error| {
            anyhow::anyhow!(
                "Failed to read voxel definitions directory {}: {}",
                definitions_path.display(),
                error
            )
        })?;

        let mut definitions_json = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!("Failed to read directory entry: {error}");
                    continue;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                warn!("Skipping non-JSON file: {}", path.display());
                continue;
            }

            match std::fs::read_to_string(&path) {
                Ok(contents) => match serde_json::from_str::<VoxelDefinitionJson>(&contents) {
                    Ok(definition) => {
                        info!(
                            "Loaded voxel definition: {} from {}",
                            definition.voxel_type,
                            path.display()
                        );
                        definitions_json.push(definition);
                    }
                    Err(error) => {
                        error!("Failed to parse JSON from {}: {}", path.display(), error);
                    }
                },
                Err(error) => {
                    error!("Failed to read file {}: {}", path.display(), error);
                }
            }
        }

        if definitions_json.is_empty() {
            warn!("No voxel definitions loaded");
            return Ok(registry);
        }

        definitions_json.sort_by(|a, b| a.texture.cmp(&b.texture));

        let mut seen_textures = std::collections::HashSet::new();
        for definition in &definitions_json {
            if seen_textures.insert(definition.texture.clone()) {
                registry.texture_order.push(definition.texture.clone());
            }
        }

        info!("Texture order for atlas: {:?}", registry.texture_order);

        for (id, def_json) in definitions_json.into_iter().enumerate() {
            let id = id as u16;
            let voxel_type = def_json.voxel_type.clone();

            let definition = VoxelDefinition::from_json(def_json, (0.0, 0.0, 1.0, 1.0));

            registry.type_to_id.insert(voxel_type.clone(), id);
            registry.id_to_definition.insert(id, definition);
        }

        info!("Loaded {} voxel types", registry.type_to_id.len());

        Ok(registry)
    }

    pub(crate) fn update_texture_coordinates(&mut self, texture_widths: &[u32], atlas_width: u32) {
        let mut texture_offset_map = HashMap::new();
        let mut x_offset = 0;

        for (texture_name, &width) in self.texture_order.iter().zip(texture_widths.iter()) {
            let u_min = x_offset as f32 / atlas_width as f32;
            let u_max = (x_offset + width) as f32 / atlas_width as f32;

            texture_offset_map.insert(texture_name.clone(), (u_min, u_max));
            x_offset += width;
        }

        let mut ids = self.id_to_definition.keys().copied().collect::<Vec<u16>>();
        ids.sort_unstable();

        for id in ids {
            let definition = self.id_to_definition.get_mut(&id).expect("ID should exist");
            if let Some(&(u_min, u_max)) = texture_offset_map.get(&definition.texture_name) {
                definition.texture_coordinates = (u_min, 0.0, u_max, 1.0);
                info!(
                    "Set texture coordinates for {}: ({}, 0.0, {}, 1.0)",
                    definition.voxel_type, u_min, u_max
                );
            } else {
                warn!(
                    "Texture '{}' not found in atlas for voxel type '{}'",
                    definition.texture_name, definition.voxel_type
                );
            }
        }
    }

    pub(crate) fn texture_order(&self) -> &[String] {
        &self.texture_order
    }

    pub(crate) fn get_definition(&self, id: u16) -> Option<&VoxelDefinition> {
        self.id_to_definition.get(&id)
    }

    pub(crate) fn get_id(&self, voxel_type: &str) -> Option<u16> {
        self.type_to_id.get(voxel_type).copied()
    }
}
