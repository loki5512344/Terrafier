# Changelog

## [0.1.0] - 2026-05-27

### Added
- Core world model (World, Dimension, Tile, Terrain, Platform)
- NBT reader/writer with full tag support (Byte, Short, Int, etc.)
- Anvil region (.mca) format reader with chunk parsing
- Block palette compression (BitArray, BlockPalette, SectionData)
- Biome database with 62 Minecraft 1.21 biomes and colour mapping
- CLI commands: new, import, export, info, render
- GUI application with egui (world viewport, tools, undo/redo)
- Coordinate system with explicit conversion functions
- Vector brush and Operation trait for editing
- Height map generation with noise-based terrain
