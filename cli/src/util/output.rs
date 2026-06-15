use serde_json::Value;

pub enum OutputFormat {
    Json,
    Human,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "human" => Self::Human,
            _ => Self::Json,
        }
    }
}

pub fn print_result(format: &OutputFormat, result: &Value) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(result).unwrap());
        }
        OutputFormat::Human => print_human(result),
    }
}

fn print_human(result: &Value) {
    if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
        match status {
            "created" => {
                let name = result["name"].as_str().unwrap_or("?");
                let seed = result["seed"].as_u64().unwrap_or(0);
                let tiles = result["tiles"].as_u64().unwrap_or(0);
                println!(
                    "Created world '{}' (seed: {}, tiles: {})",
                    name, seed, tiles
                );
            }
            "exported" => {
                let input = result["input"].as_str().unwrap_or("?");
                let output = result["output"].as_str().unwrap_or("?");
                let tiles = result["tiles"].as_u64().unwrap_or(0);
                println!("Exported {} tiles from '{}' to '{}'", tiles, input, output);
            }
            "imported" => {
                let name = result["name"].as_str().unwrap_or("?");
                let seed = result["seed"].as_u64().unwrap_or(0);
                let dims = result["dimensions"].as_u64().unwrap_or(0);
                let tiles = result["tiles"].as_u64().unwrap_or(0);
                println!(
                    "Imported world '{}' (seed: {}, {} dimensions, {} tiles)",
                    name, seed, dims, tiles
                );
            }
            "rendered" => {
                let world = result["world"].as_str().unwrap_or("?");
                let output = result["output"].as_str().unwrap_or("?");
                println!("Rendered world '{}' to '{}'", world, output);
            }
            "validated" => {
                println!("Validation passed: {:?}", result);
            }
            _ => {
                println!("{}", serde_json::to_string_pretty(result).unwrap());
            }
        }
    } else if result.get("name").is_some() {
        // Info output
        println!("World: {}", result["name"].as_str().unwrap_or("?"));
        if let Some(seed) = result.get("seed").and_then(|v| v.as_u64()) {
            println!("  Seed: {}", seed);
        }
        if let Some(platform) = result.get("platform").and_then(|v| v.as_str()) {
            println!("  Platform: {}", platform);
        }
        if let Some(height) = result.get("height_range") {
            let min = height["min"].as_i64().unwrap_or(0);
            let max = height["max"].as_i64().unwrap_or(0);
            println!("  Height range: {} to {}", min, max);
        }
        if let Some(dims) = result.get("dimensions").and_then(|v| v.as_array()) {
            println!("  Dimensions:");
            for dim in dims {
                let name = dim["name"].as_str().unwrap_or("?");
                let tiles = dim["tiles"].as_u64().unwrap_or(0);
                println!("    - {} ({} tiles)", name, tiles);
            }
        }
        if let Some(tiles) = result.get("total_tiles").and_then(|v| v.as_u64()) {
            println!("  Total tiles: {}", tiles);
        }
        if let Some(path) = result.get("path").and_then(|v| v.as_str()) {
            println!("  Path: {}", path);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result).unwrap());
    }
}

pub fn print_error(format: &OutputFormat, error: &anyhow::Error) {
    match format {
        OutputFormat::Json => {
            eprintln!(
                "{}",
                serde_json::to_string(&serde_json::json!({"error": error.to_string()})).unwrap()
            );
        }
        OutputFormat::Human => {
            eprintln!("Error: {}", error);
        }
    }
}

#[allow(dead_code)]
pub fn print_info(format: &OutputFormat, msg: &str) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({"info": msg})).unwrap()
            );
        }
        OutputFormat::Human => {
            println!("{}", msg);
        }
    }
}
