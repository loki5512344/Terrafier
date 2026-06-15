# Terrafier

> Rust-native Minecraft world painter — быстрее, легче, современнее.

Нативная замена [WorldPainter](https://www.worldpainter.net) на Rust.
Рендеринг карт через GPU, параллельный экспорт, CLI-first архитектура.

## Архитектура

```
┌────────────────────────────────────────────────────┐
│  terrafier-cli    terrafier-gui    Библиотека       │
│  (clap + json)    (egui + eframe)  (crates.io)     │
├───────────────────┬────────────────────────────────┤
│  terrafier-core   │  модель мира, I/O, операции    │
│  ┌──────┬──────┬──┴─────┬────────┐                 │
│  │World │Tile  │Export  │Import  │                 │
│  │Model │Ops   │Pipeline│Anvil   │                 │
│  └──────┴──────┴────────┴────────┘                 │
├────────────────────────────────────────────────────┤
│  Foundation crates                                  │
│  nbt │ fastanvil │ noise │ palette-compress         │
│  biome-db                                          │
└────────────────────────────────────────────────────┘
```

## Текущий статус (v0.1.0)

### Реализовано
- **Модель мира**: World, Dimension, Tile (128×128), Terrain (7 типов), Layer trait, Platform
- **Система координат**: явные конвертации BlockCoords → ChunkCoords → RegionCoords → TileCoords
- **NBT**: full read/write, все типы тегов, gzip support
- **Anvil**: чтение/запись .mca регионов, разбор чанков, секций, палитры
- **Экспорт**: Terrafier → Java Edition 1.18+ save (секции, block_states, биомы)
- **Импорт**: Java Edition save → Terrafier world (level.dat, регионы, surface)
- **CLI**: new, import, export, info, render (с прогресс-барами, dry-run)
- **GUI**: редактор на egui (viewport, инструменты, undo/redo)
- **Операции**: Raise, Lower, Smooth, Flatten, Paint — с MultiTile поддержкой
- **Heightmap**: noise-based генерация (OpenSimplex), flat, combined

### В работе / не реализовано
- **GPU-рендеринг (wgpu)** — ключевая фича, пока весь рендер CPU
- **Слои**: Caves, Rivers, Frost, Trees, Resources, Biome
- **Импорт .world** (WorldPainter format)
- **Плагины** (WASM)

## Стек

| Компонент | Технология |
|-----------|-----------|
| Язык | Rust (edition 2024) |
| GUI | egui + eframe |
| GPU | (план) wgpu |
| NBT | Самописный `nbt` crate |
| Anvil | Самописный `fastanvil` crate |
| Шум | `noise-rs` |
| Параллелизм | rayon |
| CLI | clap + indicatif |
| Изображения | `image` crate |
| Сериализация | serde + bincode |

### Почему egui, а не Tauri

egui выбран за:
- Immediate mode — нет оверхэда как у React, подходит для тулов
- Нативная интеграция с Rust (без bridge)
- wgpu-рендеринг напрямую (когда будет реализован)
- Бинарник 2–3 MB vs Tauri 5–10 MB + WebView

## Roadmap

### Phase 0 (done) — Foundation
NBT парсер, Anvil reader, workspace setup

### Phase 1 (done) — Core Model
Tile, Dimension, World, Terrain, Brush, Operation

### Phase 2 (done) — Minecraft I/O + CLI
Импорт/экспорт Java Edition, CLI команды

### Phase 3 — Operations & Layers
Инструменты (raise, erode, smooth, flatten, fill, paint)
Слои (caves, river, frost, trees, biome, resources)
Экспортёры слоёв

### Phase 4 — GUI
GPU-рендеринг через wgpu (шейдеры, 30-60 FPS)
Панорамирование, зум, оверлей кисти
Панель слоёв, инструментов, диалоги

### Phase 5 — Полировка
Бенчмарки, оптимизация, i18n, пакеты (AppImage/msi/app)

### Будущее
Импорт .world, WASM плагины, Bedrock Edition, скриптинг

## Модель данных

```rust
struct Tile {                    // 128×128 блоков
    heightmap: [i16; 16384],     // карта высот
    terrain: [u8; 16384],        // тип террейна
    water_level: [u8; 16384],    // уровень воды
    layer_data: HashMap<u32, LayerBuffer>,
}

struct Dimension {
    tiles: HashMap<(i32, i32), Tile>,
    min_height, max_height: i16,
    seed: u64,
}

struct World {
    name: String,
    dimensions: Vec<Dimension>,
    platform: Platform,
    seed: u64,
}
```
