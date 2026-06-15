//! Editing operations — raise, lower, smooth, flatten, etc.
//!
//! Each operation implements the Operation trait and supports undo.

use std::sync::OnceLock;
use std::sync::Arc;

use crate::model::dimension::Dimension;
use crate::model::terrain::Terrain;
use crate::model::tile::TILE_SIZE;

/// An operation that can be applied and reverted.
pub trait Operation: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError>;
    fn inverse(&self) -> Box<dyn Operation>;
}

#[derive(Debug)]
pub enum OperationError {
    InvalidParameters(String),
    OutOfBounds { tx: i32, tz: i32, x: u32, z: u32 },
}

/// Raise or lower terrain height within a brush area.
pub struct HeightOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub center_x: u32,
    pub center_z: u32,
    pub radius: u32,
    pub delta: i16,
    pub brush: Arc<dyn crate::model::brush::Brush>,
    pub before_snapshot: OnceLock<Vec<(usize, i16)>>,
}

impl Operation for HeightOperation {
    fn name(&self) -> &'static str {
        if self.delta >= 0 {
            "Raise"
        } else {
            "Lower"
        }
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;

        let r = self.radius as i32;
        let cx = self.center_x as i32;
        let cz = self.center_z as i32;

        let mut heights: Vec<(usize, i16)> = Vec::new();

        for dz in -r..=r {
            for dx in -r..=r {
                let ax = cx + dx;
                let az = cz + dz;
                if ax < 0 || az < 0 || ax >= TILE_SIZE as i32 || az >= TILE_SIZE as i32 {
                    continue;
                }
                let strength = self.brush.get_strength(dx as f64, dz as f64);
                if strength <= 0.0 {
                    continue;
                }

                let idx = (az as usize) * TILE_SIZE + (ax as usize);

                // Save original height
                if self.before_snapshot.get().is_none() {
                    heights.push((idx, tile.heightmap[idx]));
                }

                let change = (self.delta as f64 * strength).round() as i16;
                let new_height =
                    (tile.heightmap[idx] as i16 + change).clamp(tile.min_height, tile.max_height);
                tile.heightmap[idx] = new_height;
            }
        }

        if self.before_snapshot.get().is_none() {
            let _ = self.before_snapshot.set(heights);
        }

        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        let snapshot = self.before_snapshot.get()
            .cloned()
            .unwrap_or_default();
        Box::new(RestoreHeightsOperation {
            tile_x: self.tile_x,
            tile_z: self.tile_z,
            snapshot,
        })
    }
}

/// Flatten terrain to a target height within a brush area.
pub struct FlattenOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub center_x: u32,
    pub center_z: u32,
    pub radius: u32,
    pub target_height: i16,
    pub brush: Arc<dyn crate::model::brush::Brush>,
    pub before_snapshot: OnceLock<Vec<(usize, i16)>>,
}

impl Operation for FlattenOperation {
    fn name(&self) -> &'static str {
        "Flatten"
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;

        let r = self.radius as i32;
        let cx = self.center_x as i32;
        let cz = self.center_z as i32;

        // Phase 1: collect (idx, original_height) snapshot + apply modifications
        let mut heights: Vec<(usize, i16)> = Vec::new();

        for dz in -r..=r {
            for dx in -r..=r {
                let ax = cx + dx;
                let az = cz + dz;
                if ax < 0 || az < 0 || ax >= TILE_SIZE as i32 || az >= TILE_SIZE as i32 {
                    continue;
                }
                let strength = self.brush.get_strength(dx as f64, dz as f64);
                if strength <= 0.0 {
                    continue;
                }

                let idx = (az as usize) * TILE_SIZE + (ax as usize);

                // Save original height for undo
                if self.before_snapshot.get().is_none() {
                    heights.push((idx, tile.heightmap[idx]));
                }

                let current = tile.heightmap[idx];
                let diff = self.target_height as i32 - current as i32;
                let change = (diff as f64 * strength).round() as i32;
                let new_height = (current as i32 + change)
                    .clamp(tile.min_height as i32, tile.max_height as i32)
                    as i16;
                tile.heightmap[idx] = new_height;
            }
        }

        // Store snapshot on first call
        if self.before_snapshot.get().is_none() {
            let _ = self.before_snapshot.set(heights);
        }

        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        let snapshot = self.before_snapshot.get()
            .cloned()
            .unwrap_or_default();
        Box::new(RestoreHeightsOperation {
            tile_x: self.tile_x,
            tile_z: self.tile_z,
            snapshot,
        })
    }
}

/// Paint terrain type within a brush area.
pub struct PaintOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub center_x: u32,
    pub center_z: u32,
    pub radius: u32,
    pub terrain: Terrain,
    pub brush: Arc<dyn crate::model::brush::Brush>,
    pub before_snapshot: OnceLock<Vec<(usize, u8)>>,
}

impl Operation for PaintOperation {
    fn name(&self) -> &'static str {
        "Paint"
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;

        let r = self.radius as i32;
        let cx = self.center_x as i32;
        let cz = self.center_z as i32;
        let terrain_id = self.terrain as u8;

        let mut snapshot: Vec<(usize, u8)> = Vec::new();

        for dz in -r..=r {
            for dx in -r..=r {
                let ax = cx + dx;
                let az = cz + dz;
                if ax < 0 || az < 0 || ax >= TILE_SIZE as i32 || az >= TILE_SIZE as i32 {
                    continue;
                }
                if self.brush.get_strength(dx as f64, dz as f64) > 0.0 {
                    let idx = (az as usize) * TILE_SIZE + (ax as usize);
                    if self.before_snapshot.get().is_none() {
                        snapshot.push((idx, tile.terrain[idx]));
                    }
                    tile.terrain[idx] = terrain_id;
                }
            }
        }

        if self.before_snapshot.get().is_none() {
            let _ = self.before_snapshot.set(snapshot);
        }

        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        let snapshot = self.before_snapshot.get()
            .cloned()
            .unwrap_or_default();
        Box::new(RestoreTerrainOperation {
            tile_x: self.tile_x,
            tile_z: self.tile_z,
            snapshot,
        })
    }
}

/// Restores heights from a saved snapshot (used as inverse of FlattenOperation).
pub struct RestoreHeightsOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub snapshot: Vec<(usize, i16)>,
}

impl Operation for RestoreHeightsOperation {
    fn name(&self) -> &'static str {
        "RestoreHeights"
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;
        for &(idx, h) in &self.snapshot {
            if idx < tile.heightmap.len() {
                tile.heightmap[idx] = h;
            }
        }
        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        Box::new(NoOpOperation)
    }
}

/// Restores terrain values from a saved snapshot (used as inverse of PaintOperation).
pub struct RestoreTerrainOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub snapshot: Vec<(usize, u8)>,
}

impl Operation for RestoreTerrainOperation {
    fn name(&self) -> &'static str {
        "RestoreTerrain"
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;
        for &(idx, t) in &self.snapshot {
            if idx < tile.terrain.len() {
                tile.terrain[idx] = t;
            }
        }
        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        Box::new(NoOpOperation)
    }
}

/// Smooth terrain by averaging heights in a 3x3 neighborhood.
pub struct SmoothOperation {
    pub tile_x: i32,
    pub tile_z: i32,
    pub center_x: u32,
    pub center_z: u32,
    pub radius: u32,
    pub iterations: u32,
    pub brush: Arc<dyn crate::model::brush::Brush>,
    pub before_snapshot: OnceLock<Vec<(usize, i16)>>,
}

impl Operation for SmoothOperation {
    fn name(&self) -> &'static str {
        "Smooth"
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        let tile =
            dim.tiles
                .get_mut(&(self.tile_x, self.tile_z))
                .ok_or(OperationError::OutOfBounds {
                    tx: self.tile_x,
                    tz: self.tile_z,
                    x: 0,
                    z: 0,
                })?;

        let r = self.radius as i32;
        let cx = self.center_x as i32;
        let cz = self.center_z as i32;

        // Snapshot original heights
        let mut snapshot: Vec<(usize, i16)> = Vec::new();
        for dz in -r..=r {
            for dx in -r..=r {
                let ax = cx + dx;
                let az = cz + dz;
                if ax < 0 || az < 0 || ax >= TILE_SIZE as i32 || az >= TILE_SIZE as i32 {
                    continue;
                }
                if self.brush.get_strength(dx as f64, dz as f64) <= 0.0 {
                    continue;
                }
                if self.before_snapshot.get().is_none() {
                    snapshot.push((
                        (az as usize) * TILE_SIZE + (ax as usize),
                        tile.heightmap[(az as usize) * TILE_SIZE + (ax as usize)],
                    ));
                }
            }
        }
        if self.before_snapshot.get().is_none() {
            let _ = self.before_snapshot.set(snapshot);
        }

        // Apply smoothing for `iterations` passes
        for _ in 0..self.iterations.max(1) {
            // Copy current heights into temp buffer
            let mut new_heights = tile.heightmap;

            for dz in -r..=r {
                for dx in -r..=r {
                    let ax = cx + dx;
                    let az = cz + dz;
                    if ax < 0 || az < 0 || ax >= TILE_SIZE as i32 || az >= TILE_SIZE as i32 {
                        continue;
                    }
                    if self.brush.get_strength(dx as f64, dz as f64) <= 0.0 {
                        continue;
                    }

                    let idx = (az as usize) * TILE_SIZE + (ax as usize);

                    // Average of 3x3 neighborhood (clamped to tile bounds)
                    let mut sum = 0i32;
                    let mut count = 0i32;
                    for ny in -1..=1i32 {
                        for nx in -1..=1i32 {
                            let bx = ax + nx;
                            let bz = az + ny;
                            if bx >= 0 && bx < TILE_SIZE as i32 && bz >= 0 && bz < TILE_SIZE as i32
                            {
                                sum +=
                                    tile.heightmap[(bz as usize) * TILE_SIZE + (bx as usize)] as i32;
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        new_heights[idx] = (sum / count)
                            .clamp(tile.min_height as i32, tile.max_height as i32)
                            as i16;
                    }
                }
            }

            tile.heightmap = new_heights;
        }

        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        let snapshot = self.before_snapshot.get().cloned().unwrap_or_default();
        Box::new(RestoreHeightsOperation {
            tile_x: self.tile_x,
            tile_z: self.tile_z,
            snapshot,
        })
    }
}

/// Applies multiple sub-operations, supporting brushes that span multiple tiles.
pub struct MultiTileOperation {
    pub operations: Vec<Box<dyn Operation>>,
}

impl Operation for MultiTileOperation {
    fn name(&self) -> &'static str {
        if self.operations.is_empty() {
            "MultiTile (empty)"
        } else {
            self.operations[0].name()
        }
    }

    fn apply(&self, dim: &mut Dimension) -> Result<(), OperationError> {
        for op in &self.operations {
            op.apply(dim)?;
        }
        Ok(())
    }

    fn inverse(&self) -> Box<dyn Operation> {
        let mut inverses: Vec<Box<dyn Operation>> = Vec::with_capacity(self.operations.len());
        for op in &self.operations {
            inverses.push(op.inverse());
        }
        inverses.reverse();
        Box::new(MultiTileOperation {
            operations: inverses,
        })
    }
}

/// No-op operation (used as fallback inverse).
pub struct NoOpOperation;

impl Operation for NoOpOperation {
    fn name(&self) -> &'static str {
        "NoOp"
    }
    fn apply(&self, _dim: &mut Dimension) -> Result<(), OperationError> {
        Ok(())
    }
    fn inverse(&self) -> Box<dyn Operation> {
        Box::new(NoOpOperation)
    }
}
