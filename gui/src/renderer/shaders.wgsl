struct Uniforms {
    min_height: i32,
    max_height: i32,
    show_heightmap: u32,
    tile_size: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> heightmap_packed: array<u32>;
@group(0) @binding(2) var<storage, read> terrain_packed: array<u32>;
@group(0) @binding(3) var<storage, read_write> output: array<u32>;

fn unpack_heightmap(data: array<u32>, index: u32) -> i32 {
    let packed = data[index / 2u];
    var val: i32;
    if index % 2u == 0u {
        val = i32(packed & 0xFFFFu);
    } else {
        val = i32(packed >> 16u);
    }
    if val & 0x8000 != 0 {
        val = val | 0xFFFF0000;
    }
    return val;
}

fn unpack_terrain(data: array<u32>, index: u32) -> u32 {
    let packed = data[index / 4u];
    let shift = (index % 4u) * 8u;
    return (packed >> shift) & 0xFFu;
}

fn terrain_color(id: u32, height_factor: f32) -> vec3<f32> {
    var color: vec3<f32>;
    switch id {
        case 0u: {
            color = vec3<f32>(194.0, 178.0, 128.0);
        }
        case 1u: {
            color = vec3<f32>(124.0, 189.0, 107.0);
        }
        case 2u: {
            color = vec3<f32>(86.0, 140.0, 74.0);
        }
        case 3u: {
            color = vec3<f32>(128.0, 128.0, 128.0);
        }
        case 4u: {
            color = vec3<f32>(227.0, 212.0, 160.0);
        }
        case 5u: {
            color = vec3<f32>(72.0, 107.0, 75.0);
        }
        case 6u: {
            color = vec3<f32>(64.0, 128.0, 255.0);
        }
        default: {
            color = vec3<f32>(128.0, 128.0, 128.0);
        }
    }
    return color * height_factor;
}

fn height_color(norm: f32) -> vec3<f32> {
    var color: vec3<f32>;
    if norm < 0.25 {
        let t = norm / 0.25;
        color = mix(vec3<f32>(0.0, 0.0, 255.0), vec3<f32>(0.0, 255.0, 255.0), t);
    } else if norm < 0.5 {
        let t = (norm - 0.25) / 0.25;
        color = mix(vec3<f32>(0.0, 255.0, 255.0), vec3<f32>(0.0, 255.0, 0.0), t);
    } else if norm < 0.75 {
        let t = (norm - 0.5) / 0.25;
        color = mix(vec3<f32>(0.0, 255.0, 0.0), vec3<f32>(255.0, 255.0, 0.0), t);
    } else {
        let t = (norm - 0.75) / 0.25;
        color = mix(vec3<f32>(255.0, 255.0, 0.0), vec3<f32>(255.0, 0.0, 0.0), t);
    }
    return color;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= uniforms.tile_size || id.y >= uniforms.tile_size {
        return;
    }

    let index = id.y * uniforms.tile_size + id.x;

    let height = unpack_heightmap(heightmap_packed, index);
    let terrain_id = unpack_terrain(terrain_packed, index);

    let height_factor = 0.7 + 0.3 * ((f32(height) + 64.0) / 384.0);

    var color: vec3<f32>;
    if uniforms.show_heightmap != 0u {
        let height_range = f32(uniforms.max_height - uniforms.min_height);
        var norm: f32;
        if height_range <= 0.0 {
            norm = 0.5;
        } else {
            norm = (f32(height) - f32(uniforms.min_height)) / height_range;
        }
        norm = clamp(norm, 0.0, 1.0);
        color = height_color(norm);
    } else {
        color = terrain_color(terrain_id, height_factor);
    }

    output[index] = (u32(color.r) & 0xFFu) |
                    ((u32(color.g) & 0xFFu) << 8u) |
                    ((u32(color.b) & 0xFFu) << 16u) |
                    0xFF000000u;
}
