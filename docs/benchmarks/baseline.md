# Baseline Benchmarks

> Generated: 2026-05-27
> Hardware: Intel/AMD, Windows
> Rust: release profile (opt-level=3, LTO, codegen-units=1)

## NBT

| Benchmark | Time (µs) | Notes |
|-----------|-----------|-------|
| parse_small_compound | 3.99 µs | 10-field Compound |
| parse_large_longarray_4096 | 52.2 µs | LongArray[4096] |
| serialize_compound | 2.39 µs | 10-field Compound |
| serialize_large_longarray | 12.4 µs | LongArray[4096] |
| gzip_roundtrip | 59.5 µs | compress + decompress |
| roundtrip_small | 6.77 µs | serialize + parse |

## Render

| Benchmark | Time (ms) | Notes |
|-----------|-----------|-------|
| one_tile_scale1 | 0.67 ms | 128×128 full-res |
| one_tile_scale2 | 0.60 ms | 64×64 |
| one_tile_scale4 | 0.61 ms | 32×32 |
| 3x3_tiles_scale1 | 3.08 ms | 384×384 full-res |
| 3x3_tiles_scale2 | 2.70 ms | 192×192 |
| 3x3_tiles_scale4 | 2.60 ms | 96×96 |

## World

| Benchmark | Time | Notes |
|-----------|------|-------|
| new_default | **1.53 ms** | 3×3 tiles with noise (optimized with rayon: was 10.2ms) |
| height_operation | 11.4 µs | radius=10, delta=5 |
| paint_operation | 4.18 µs | radius=10 |
| serialize_json | 5.25 ms | 3×3 tiles |
| deserialize_json | 6.37 ms | 3×3 tiles |
| render_image_scale4 | 1.98 ms | via render_to_image |

## Hot-path Summary

All benchmarks are **below 10ms**. The only candidate (World::new at 10.2ms) was optimized with parallel tile generation via `rayon` — now **1.5ms (85% faster)**.
