# Hot-spot Analysis

> Updated: 2026-05-27

## Results

After baseline measurement, **only 1 benchmark exceeded 10ms**:

| Benchmark | Baseline | Optimized | Improvement |
|-----------|----------|-----------|-------------|
| `world/new_default` | **10.2 ms** | **1.53 ms** | **85%** |

All other benchmarks are well within the threshold (<7ms).

## Optimisation Applied

### `World::new()` — parallel tile generation

**Problem**: `World::new()` generated 9 noise heightmaps sequentially in nested `for` loops
(`-1..=1 × -1..=1`). Each tile calls `NoiseHeightMap::generate()` which does 16384 noise
samples × noise function calls. Total: 9 × 16384 = 147,456 noise evaluations.

**Fix**: Replaced sequential loop with `rayon::par_iter()` over the tile coordinates.
Noise heightmap generation per tile is embarrassingly parallel — tiles are fully independent.

**Result**: 10.2ms → 1.53ms (6.7x speedup on 8-core machine).

## Other Candidates (no action needed)

### NBT parsing (max 52 µs)
No action needed — already below 100µs. Zero-copy optimisations would add complexity
with negligible benefit.

### Render (max 3.1 ms)
No action needed — pixel-by-pixel rendering at 3ms for 384×384 is acceptable for a
preview renderer. If tile count grows to 100+, chunk-level parallelism via rayon
would help. Not needed now.

### JSON serialization (max 5.3 ms / 6.4 ms)
Bottleneck is serde serializing 9 tiles × 16384-element arrays. Potential future optimisations:
- Streaming serialization with `serde_json::to_writer`
- Custom serializer using binary formats
- Compressed tile data representation

Not needed now — user-facing operations (`new`, `paint`, `height` edits) are sub-millisecond.
