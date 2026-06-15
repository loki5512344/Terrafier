//! Minecraft world I/O — read and write Java Edition saves.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use log;

use terrafier_fastanvil::io::region::Region;
use terrafier_nbt::io::reader::read_gzip;

use crate::model::dimension::Dimension;
use crate::model::platform::Platform;
use crate::model::tile::Tile;
use crate::model::world::World;

#[derive(Error, Debug)]
pub enum MinecraftIOError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("NBT parse error: {0}")]
    NbtParse(#[from] terrafier_nbt::io::reader::ReadError),
    #[error("NBT write error: {0}")]
    NbtWrite(#[from] terrafier_nbt::io::writer::WriteError),
    #[error("Region error: {0}")]
    Region(#[from] terrafier_fastanvil::io::region::RegionError),
    #[error("Not a Minecraft save directory: {0}")]
    NotASave(String),
    #[error("Missing level.dat")]
    MissingLevelDat,
    #[error("Missing region directory")]
    MissingRegionDir,
    #[error("Unsupported data version: {0}")]
    UnsupportedVersion(i32),
    #[error("Invalid chunk data: {0}")]
    InvalidChunk(String),
}

pub type Result<T> = std::result::Result<T, MinecraftIOError>;

/// Infer Minecraft version from DataVersion number.
pub fn version_from_data_version(dv: i32) -> Option<Platform> {
    match dv {
        2860..=2865 => Some(Platform {
            id: "java_1_18".into(),
            display_name: "Minecraft Java 1.18".into(),
            min_height: -64,
            max_height: 320,
        }),
        2866..=2974 => {
            log::warn!("DataVersion {} is between 1.18 and 1.19, falling back to 1.18", dv);
            Some(Platform {
                id: "java_1_18".into(),
                display_name: "Minecraft Java 1.18".into(),
                min_height: -64,
                max_height: 320,
            })
        },
        2975..=3117 => Some(Platform {
            id: "java_1_19".into(),
            display_name: "Minecraft Java 1.19".into(),
            min_height: -64,
            max_height: 320,
        }),
        3118..=3336 => {
            log::warn!("DataVersion {} is between 1.19 and 1.20, falling back to 1.19", dv);
            Some(Platform {
                id: "java_1_19".into(),
                display_name: "Minecraft Java 1.19".into(),
                min_height: -64,
                max_height: 320,
            })
        },
        3337..=3460 => Some(Platform {
            id: "java_1_20".into(),
            display_name: "Minecraft Java 1.20".into(),
            min_height: -64,
            max_height: 320,
        }),
        3461..=3577 => {
            log::warn!("DataVersion {} is between 1.20 and 1.20.5, falling back to 1.20", dv);
            Some(Platform {
                id: "java_1_20".into(),
                display_name: "Minecraft Java 1.20 (fallback)".into(),
                min_height: -64,
                max_height: 320,
            })
        },
        3578..=3700 => Some(Platform {
            id: "java_1_20_5".into(),
            display_name: "Minecraft Java 1.20.5+".into(),
            min_height: -64,
            max_height: 320,
        }),
        3701..=3818 => {
            log::warn!("DataVersion {} is between 1.20.5 and 1.21, falling back to 1.20.5", dv);
            Some(Platform {
                id: "java_1_20_5".into(),
                display_name: "Minecraft Java 1.20.5 (fallback)".into(),
                min_height: -64,
                max_height: 320,
            })
        },
        3819..=3953 => Some(Platform {
            id: "java_1_21".into(),
            display_name: "Minecraft Java 1.21".into(),
            min_height: -64,
            max_height: 320,
        }),
        3954..=4100 => Some(Platform {
            id: "java_1_21_2".into(),
            display_name: "Minecraft Java 1.21.2+".into(),
            min_height: -64,
            max_height: 320,
        }),
        _ => None,
    }
}

/// Load a Minecraft save directory into a Terrafier World model.
pub fn load_save(path: &Path) -> Result<World> {
    if !path.is_dir() {
        return Err(MinecraftIOError::NotASave(path.display().to_string()));
    }

    // Read level.dat
    let level_dat_path = path.join("level.dat");
    if !level_dat_path.exists() {
        return Err(MinecraftIOError::MissingLevelDat);
    }
    let level_data = fs::read(&level_dat_path)?;
    let level_tag = read_gzip(&level_data)?;

    let (world_name, seed, data_version) = parse_level_dat(&level_tag);
    let platform = version_from_data_version(data_version)
        .ok_or(MinecraftIOError::UnsupportedVersion(data_version))?;

    // Read region files
    let region_dir = path.join("region");
    if !region_dir.is_dir() {
        return Err(MinecraftIOError::MissingRegionDir);
    }

    let mut tiles: HashMap<(i32, i32), Tile> = HashMap::new();

    let region_entries = fs::read_dir(&region_dir)?;
    for entry in region_entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "mca") {
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let parts: Vec<&str> = file_name.split('.').collect();
            if parts.len() >= 3 {
                if let (Ok(rx), Ok(rz)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                    let region_bytes = fs::read(&path)?;
                    let region = Region::from_bytes(rx, rz, &region_bytes)?;
                    for (local_x, local_z) in region.chunk_coords() {
                        let chunk_data = region.get_chunk_data(local_x, local_z).unwrap();
                        if let Ok(chunk_tag) = read_gzip(chunk_data) {
                            if let Some(chunk) =
                                terrafier_fastanvil::io::chunk::Chunk::from_nbt(&chunk_tag)
                            {
                                let tile_x = chunk.x >> 3;
                                let tile_z = chunk.z >> 3;

                                // 8 chunks per tile (128 blocks / 16 blocks per chunk)
                                let chunk_local_x = (chunk.x & 7) as usize;
                                let chunk_local_z = (chunk.z & 7) as usize;

                                let tile = tiles
                                    .entry((tile_x, tile_z))
                                    .or_insert_with(|| Tile::new(
                                        tile_x,
                                        tile_z,
                                        platform.min_height,
                                        platform.max_height,
                                    ));

                                for lx in 0..16usize {
                                    for lz in 0..16usize {
                                        let mut surface_y = None;

                                        if !chunk.sections.is_empty() {
                                            let mut sorted: Vec<_> =
                                                chunk.sections.iter().collect();
                                            sorted
                                                .sort_by(|a, b| b.section_y.cmp(&a.section_y));

                                            for section in &sorted {
                                                if section.palette.is_empty() {
                                                    continue;
                                                }

                                                let has_blocks =
                                                    section.palette.iter().any(|p| {
                                                        p.get("Name").map_or(false, |n| {
                                                            matches!(
                                                                n,
                                                                terrafier_nbt::Tag::String(s)
                                                                    if s != "minecraft:air"
                                                            )
                                                        })
                                                    });

                                                if !has_blocks {
                                                    continue;
                                                }

                                                if section.block_data.is_empty() {
                                                    if section.palette[0]
                                                        .get("Name")
                                                        .map_or(false, |n| {
                                                            matches!(
                                                                n,
                                                                terrafier_nbt::Tag::String(s)
                                                                    if s == "minecraft:air"
                                                            )
                                                        })
                                                    {
                                                        continue;
                                                    }
                                                    surface_y = Some(
                                                        (section.section_y as i32) * 16 + 15,
                                                    );
                                                    break;
                                                }

                                                surface_y = Some(
                                                    (section.section_y as i32) * 16 + 15,
                                                );
                                                break;
                                            }
                                        }

                                        let tile_local_x = chunk_local_x * 16 + lx;
                                        let tile_local_z = chunk_local_z * 16 + lz;

                                        if tile_local_x < 128 && tile_local_z < 128 {
                                            if let Some(y) = surface_y {
                                                let clamped = (y as i16)
                                                    .clamp(tile.min_height, tile.max_height);
                                                tile.heightmap
                                                    [tile_local_z * 128 + tile_local_x] = clamped;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let dimension = Dimension {
        name: "overworld".to_string(),
        tiles,
        min_height: platform.min_height,
        max_height: platform.max_height,
        seed,
    };

    Ok(World {
        name: world_name,
        platform,
        dimensions: vec![dimension],
        seed,
    })
}

/// Save a Terrafier World to a Minecraft save directory.
pub fn save_world(world: &World, output_path: &Path) -> Result<()> {
    use std::collections::BTreeMap;

    fs::create_dir_all(output_path.join("region"))?;

    // Write level.dat
    let level_tag = build_level_dat(world)?;
    let level_bytes = terrafier_nbt::io::writer::to_gzip_bytes(&level_tag)?;
    fs::write(output_path.join("level.dat"), &level_bytes)?;

    let dim = match world.dimensions.first() {
        Some(d) => d,
        None => return Ok(()),
    };

    // Group tiles by region (512×512 blocks = 4×4 tiles of 128×128 each)
    let mut regions: BTreeMap<(i32, i32), Vec<(&(i32, i32), &Tile)>> = BTreeMap::new();
    for (key, tile) in &dim.tiles {
        let (tx, tz) = key;
        let rx = tx >> 2;
        let rz = tz >> 2;
        regions.entry((rx, rz)).or_default().push((key, tile));
    }

    for ((rx, rz), tile_refs) in &regions {
        let mut region = terrafier_fastanvil::io::region::Region::new(*rx, *rz);

        for (_key, tile) in tile_refs {
            // Each tile is 128x128 blocks = 8x8 chunks
            for chunk_lx in 0..8usize {
                for chunk_lz in 0..8usize {
                    let chunk_x = tile.x * 8 + chunk_lx as i32;
                    let chunk_z = tile.z * 8 + chunk_lz as i32;

                    let region_local_x = (chunk_x & 31) as u8;
                    let region_local_z = (chunk_z & 31) as u8;

                    let chunk_data = build_chunk_nbt(chunk_x, chunk_z, tile, chunk_lx, chunk_lz)?;

                    // Skip completely empty chunks (no blocks)
                    if chunk_data.is_empty() {
                        continue;
                    }

                    region.set_chunk_data(region_local_x, region_local_z, chunk_data);
                }
            }
        }

        let region_bytes = region.to_bytes()?;
        let file_name = format!("r.{}.{}.mca", rx, rz);
        fs::write(output_path.join("region").join(&file_name), &region_bytes)?;
    }

    Ok(())
}

// ---- Chunk building helpers ----

/// Minimum number of bits needed to represent values up to n-1 (clamped to MC minimum 4).
fn bits_needed(n: usize) -> usize {
    if n <= 1 {
        return 4;
    }
    let bits = (usize::BITS - (n - 1).leading_zeros()) as usize;
    bits.max(4)
}

/// Pack palette indices into a compact long array (Minecraft block state format).
fn pack_indices(indices: &[u16], bits: usize) -> Vec<i64> {
    if indices.is_empty() || bits == 0 {
        return Vec::new();
    }
    let total_bits = indices.len() * bits;
    let longs = (total_bits + 63) / 64;
    let mut data = vec![0i64; longs];
    let mask = (1i64 << bits) - 1;
    for (i, &idx) in indices.iter().enumerate() {
        let bit_pos = i * bits;
        let long_idx = bit_pos / 64;
        let bit_offset = bit_pos % 64;
        data[long_idx] |= (idx as i64 & mask) << bit_offset;
    }
    data
}

/// Determine the Minecraft block at a given (global_y) column position.
fn block_name(terrain: u8, y: i32, surface_y: i32) -> &'static str {
    // Water terrain: everything up to the surface is water
    if terrain == 6 {
        if y <= surface_y {
            return "minecraft:water";
        } else {
            return "minecraft:air";
        }
    }

    // Non-water terrain
    if y == surface_y {
        match terrain {
            1 | 4 => "minecraft:sand",
            3 => "minecraft:stone",
            _ => "minecraft:grass_block",
        }
    } else if y > surface_y {
        "minecraft:air"
    } else if y > surface_y - 4 {
        match terrain {
            1 | 4 => "minecraft:sand",
            3 => "minecraft:stone",
            _ => "minecraft:dirt",
        }
    } else if y < -60 {
        "minecraft:bedrock"
    } else if y < 0 {
        "minecraft:deepslate"
    } else {
        "minecraft:stone"
    }
}

/// Build a single chunk's NBT data (raw uncompressed bytes).
/// Returns empty Vec if the chunk has no blocks at all.
fn build_chunk_nbt(
    chunk_x: i32,
    chunk_z: i32,
    tile: &Tile,
    chunk_lx: usize,
    chunk_lz: usize,
) -> Result<Vec<u8>> {
    let mut compound = HashMap::new();
    compound.insert("xPos".into(), terrafier_nbt::Tag::Int(chunk_x));
    compound.insert("zPos".into(), terrafier_nbt::Tag::Int(chunk_z));
    compound.insert("DataVersion".into(), terrafier_nbt::Tag::Int(3954));
    compound.insert("Status".into(), terrafier_nbt::Tag::String("full".into()));

    // Palette of all blocks used in this chunk
    const BLOCK_SET: &[&str] = &[
        "minecraft:air",
        "minecraft:grass_block",
        "minecraft:dirt",
        "minecraft:stone",
        "minecraft:bedrock",
        "minecraft:sand",
        "minecraft:sandstone",
        "minecraft:water",
        "minecraft:deepslate",
    ];

    let min_sec = (tile.min_height as i32 >> 4).max(-4);
    let max_sec = (tile.max_height as i32 >> 4).min(20);

    let mut sections: Vec<terrafier_nbt::Tag> = Vec::new();

    for sec_y in min_sec..=max_sec {
        let sec_base = sec_y * 16;

        // Collect block indices for all 4096 positions in this section
        // MC order: y * 16 * 16 + z * 16 + x = y * 256 + z * 16 + x
        let mut indices = Vec::with_capacity(4096);
        for y_rel in 0..16 {
            let global_y = sec_base + y_rel;
            for lz in 0..16 {
                for lx in 0..16 {
                    let tile_lx = chunk_lx * 16 + lx;
                    let tile_lz = chunk_lz * 16 + lz;

                    let idx = if tile_lx >= 128 || tile_lz >= 128 {
                        0u16 // air for out-of-bounds
                    } else {
                        let surface_y = tile.heightmap[tile_lz * 128 + tile_lx] as i32;
                        let terrain_id = tile.terrain[tile_lz * 128 + tile_lx];
                        let name = block_name(terrain_id, global_y, surface_y);
                        BLOCK_SET.iter().position(|s| *s == name).unwrap_or(0) as u16
                    };
                    indices.push(idx);
                }
            }
        }

        // Skip sections with only air blocks
        if !indices.iter().any(|&i| i != 0) {
            continue;
        }

        let bits = bits_needed(BLOCK_SET.len());
        let packed = pack_indices(&indices, bits);

        let palette_tags: Vec<terrafier_nbt::Tag> = BLOCK_SET.iter().map(|name| {
            let mut entry = HashMap::new();
            entry.insert("Name".into(), terrafier_nbt::Tag::String(name.to_string()));
            terrafier_nbt::Tag::Compound(entry)
        }).collect();

        let mut block_states = HashMap::new();
        block_states.insert("palette".into(), terrafier_nbt::Tag::List(palette_tags));
        block_states.insert("data".into(), terrafier_nbt::Tag::LongArray(packed));

        let biome_palette = vec![{
            let mut b = HashMap::new();
            b.insert("Name".into(), terrafier_nbt::Tag::String("minecraft:plains".into()));
            terrafier_nbt::Tag::Compound(b)
        }];
        let mut biomes = HashMap::new();
        biomes.insert("palette".into(), terrafier_nbt::Tag::List(biome_palette));

        let mut sec_compound = HashMap::new();
        sec_compound.insert("Y".into(), terrafier_nbt::Tag::Byte(sec_y as i8));
        sec_compound.insert("block_states".into(), terrafier_nbt::Tag::Compound(block_states));
        sec_compound.insert("biomes".into(), terrafier_nbt::Tag::Compound(biomes));

        sections.push(terrafier_nbt::Tag::Compound(sec_compound));
    }

    // If no sections have any blocks, return empty
    if sections.is_empty() {
        return Ok(Vec::new());
    }

    compound.insert("sections".into(), terrafier_nbt::Tag::List(sections));

    let chunk_tag = terrafier_nbt::Tag::Compound(compound);
    let bytes = terrafier_nbt::io::writer::to_bytes(&chunk_tag)?;
    Ok(bytes)
}

fn parse_level_dat(root: &terrafier_nbt::Tag) -> (String, u64, i32) {
    let default = || ("Unknown".to_string(), 0u64, 3954i32);
    match root {
        terrafier_nbt::Tag::Compound(map) => {
            let data = match map.get("Data") {
                Some(terrafier_nbt::Tag::Compound(d)) => d,
                _ => return default(),
            };
            let name = match data.get("LevelName") {
                Some(terrafier_nbt::Tag::String(s)) => s.clone(),
                _ => "Unknown".to_string(),
            };
            let seed = match data.get("WorldGenSettings").and_then(|w| match w {
                terrafier_nbt::Tag::Compound(m) => m.get("seed"),
                _ => None,
            }) {
                Some(terrafier_nbt::Tag::Long(s)) => *s as u64,
                _ => 0,
            };
            let data_version = match data.get("DataVersion") {
                Some(terrafier_nbt::Tag::Int(v)) => *v,
                _ => 3954,
            };
            (name, seed, data_version)
        }
        _ => default(),
    }
}

fn build_level_dat(world: &World) -> Result<terrafier_nbt::Tag> {
    use terrafier_nbt::Tag;
    let mut data = HashMap::new();
    data.insert("LevelName".into(), Tag::String(world.name.clone()));
    data.insert("DataVersion".into(), Tag::Int(3954));
    data.insert("version".into(), Tag::Int(19133));

    let mut world_gen = HashMap::new();
    world_gen.insert("seed".into(), Tag::Long(world.seed as i64));
    data.insert("WorldGenSettings".into(), Tag::Compound(world_gen));

    let mut root = HashMap::new();
    root.insert("Data".into(), Tag::Compound(data));

    Ok(Tag::Compound(root))
}

pub fn discover_region_files(world_path: &Path) -> Result<Vec<PathBuf>> {
    let region_dir = world_path.join("region");
    if !region_dir.is_dir() {
        return Err(MinecraftIOError::MissingRegionDir);
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&region_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "mca") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
