use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use terrafier_core::io::export::render_to_image;
use terrafier_core::model::types::{SymmetricBrush, Terrain};
use terrafier_core::model::world::World;
use terrafier_core::ops::operations::{HeightOperation, Operation, PaintOperation};

fn bench_world_new(c: &mut Criterion) {
    c.bench_function("world/new_default", |b| {
        b.iter(|| {
            let world = World::new(black_box("bench_world"), black_box(42));
            black_box(world);
        });
    });
}

fn bench_height_operation(c: &mut Criterion) {
    let mut world = World::new("bench", 42);
    let brush = Arc::new(SymmetricBrush::new(10.0));
    let op = HeightOperation {
        tile_x: 0,
        tile_z: 0,
        center_x: 64,
        center_z: 64,
        radius: 10,
        delta: 5,
        brush,
        before_snapshot: Default::default(),
    };

    c.bench_function("world/height_operation", |b| {
        b.iter(|| {
            let dim = world.overworld_mut().unwrap();
            op.apply(black_box(dim)).unwrap();
        });
    });
}

fn bench_paint_operation(c: &mut Criterion) {
    let mut world = World::new("bench", 42);
    let brush = Arc::new(SymmetricBrush::new(10.0));
    let op = PaintOperation {
        tile_x: 0,
        tile_z: 0,
        center_x: 64,
        center_z: 64,
        radius: 10,
        terrain: Terrain::Forest,
        brush,
        before_snapshot: Default::default(),
    };

    c.bench_function("world/paint_operation", |b| {
        b.iter(|| {
            let dim = world.overworld_mut().unwrap();
            op.apply(black_box(dim)).unwrap();
        });
    });
}

fn bench_world_serialize_json(c: &mut Criterion) {
    let world = World::new("bench_ser", 42);

    c.bench_function("world/serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&world)).unwrap();
            black_box(json);
        });
    });
}

fn bench_world_deserialize_json(c: &mut Criterion) {
    let world = World::new("bench_ser", 42);
    let json = serde_json::to_string(&world).unwrap();

    c.bench_function("world/deserialize_json", |b| {
        b.iter(|| {
            let w: World = serde_json::from_str(black_box(&json)).unwrap();
            black_box(w);
        });
    });
}

fn bench_world_render_image(c: &mut Criterion) {
    let world = World::new("bench_render", 42);
    let path = std::env::temp_dir().join("terrafier_bench_world.png");

    c.bench_function("world/render_image_scale4", |b| {
        b.iter(|| {
            render_to_image(black_box(&world), &path, 4).unwrap();
        });
    });
    let _ = std::fs::remove_file(&path);
}

criterion_group!(
    world,
    bench_world_new,
    bench_height_operation,
    bench_paint_operation,
    bench_world_serialize_json,
    bench_world_deserialize_json,
    bench_world_render_image
);
criterion_main!(world);
