use criterion::{Criterion, black_box, criterion_group, criterion_main};

use terrafier_core::io::export::render_to_image;
use terrafier_core::model::tile::Tile;
use terrafier_core::model::world::World;

fn make_world_with_tiles(tile_count: i32) -> World {
    let mut world = World::new("bench", 42);

    // World::new already creates 9 tiles (-1..=1, -1..=1).
    // If we need exactly 1 tile, rebuild.
    if tile_count == 1 {
        let dim = &mut world.dimensions[0];
        dim.tiles.clear();
        dim.tiles.insert(
            (0, 0),
            Tile::new(0, 0, world.platform.min_height, world.platform.max_height),
        );
    }
    world
}

fn make_full_world() -> World {
    make_world_with_tiles(9)
}

fn bench_render_one_tile_scale1(c: &mut Criterion) {
    let world = make_world_with_tiles(1);
    let path = std::env::temp_dir().join("terrafier_bench_1.png");

    c.bench_function("render/one_tile_scale1", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 1).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

fn bench_render_one_tile_scale2(c: &mut Criterion) {
    let world = make_world_with_tiles(1);
    let path = std::env::temp_dir().join("terrafier_bench_s2.png");

    c.bench_function("render/one_tile_scale2", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 2).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

fn bench_render_one_tile_scale4(c: &mut Criterion) {
    let world = make_world_with_tiles(1);
    let path = std::env::temp_dir().join("terrafier_bench_s4.png");

    c.bench_function("render/one_tile_scale4", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 4).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

fn bench_render_3x3_tiles_scale1(c: &mut Criterion) {
    let world = make_full_world();
    let path = std::env::temp_dir().join("terrafier_bench_9.png");

    c.bench_function("render/3x3_tiles_scale1", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 1).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

fn bench_render_3x3_tiles_scale2(c: &mut Criterion) {
    let world = make_full_world();
    let path = std::env::temp_dir().join("terrafier_bench_9s2.png");

    c.bench_function("render/3x3_tiles_scale2", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 2).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

fn bench_render_3x3_tiles_scale4(c: &mut Criterion) {
    let world = make_full_world();
    let path = std::env::temp_dir().join("terrafier_bench_9s4.png");

    c.bench_function("render/3x3_tiles_scale4", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 4).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

criterion_group!(
    render,
    bench_render_one_tile_scale1,
    bench_render_one_tile_scale2,
    bench_render_one_tile_scale4,
    bench_render_3x3_tiles_scale1,
    bench_render_3x3_tiles_scale2,
    bench_render_3x3_tiles_scale4
);
criterion_main!(render);
