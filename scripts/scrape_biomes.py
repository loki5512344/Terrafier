"""Scrape Minecraft 1.21 biome data from mcasset.cloud GitHub repo."""
import json
import urllib.request
import os

BIOME_NAMES = [
    "badlands", "bamboo_jungle", "basalt_deltas", "beach", "birch_forest",
    "cherry_grove", "cold_ocean", "crimson_forest", "dark_forest",
    "deep_cold_ocean", "deep_dark", "deep_frozen_ocean", "deep_lukewarm_ocean",
    "deep_ocean", "desert", "dripstone_caves", "end_barrens", "end_highlands",
    "end_midlands", "eroded_badlands", "flower_forest", "forest",
    "frozen_ocean", "frozen_peaks", "frozen_river", "grove", "ice_spikes",
    "jagged_peaks", "jungle", "lukewarm_ocean", "lush_caves",
    "mangrove_swamp", "meadow", "mushroom_fields", "nether_wastes", "ocean",
    "old_growth_birch_forest", "old_growth_pine_taiga",
    "old_growth_spruce_taiga", "pale_garden", "plains", "river", "savanna",
    "savanna_plateau", "small_end_islands", "snowy_beach", "snowy_plains",
    "snowy_slopes", "snowy_taiga", "soul_sand_valley", "sparse_jungle",
    "stony_peaks", "stony_shore", "sunflower_plains", "swamp", "taiga",
    "the_end", "the_void", "warm_ocean", "warped_forest",
    "windswept_forest", "windswept_gravelly_hills", "windswept_hills",
    "windswept_savanna", "wooded_badlands",
]

BASE_URL = "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.11/data/minecraft/worldgen/biome"


def hex_to_int(hex_str):
    """Convert '#3f76e4' to integer (0x3f76e4 = 4159204)."""
    return int(hex_str.lstrip("#"), 16)


def determine_precipitation(temperature, has_precip, name):
    """Determine precipitation type: 'rain', 'snow', or 'none'."""
    if not has_precip:
        return "none"
    if temperature <= 0.15:
        return "snow"
    return "rain"


def parse_biome(name, data):
    """Parse a Minecraft biome JSON into a flat entry."""
    effects = data.get("effects", {})
    attributes = data.get("attributes", {})
    temperature = data.get("temperature", 0.5)
    downfall = data.get("downfall", 0.5)
    has_precip = data.get("has_precipitation", True)

    # Water color from effects
    water_color = hex_to_int(effects.get("water_color", "#3f76e4"))

    # Sky color from attributes
    sky_attr = attributes.get("minecraft:visual/sky_color", "#78a7ff")
    sky_color = hex_to_int(sky_attr)

    # Fog color from attributes (Nether biomes use this)
    fog_attr = attributes.get("minecraft:visual/fog_color")
    if fog_attr:
        fog_color = hex_to_int(fog_attr)
    else:
        # Default fog color same as sky if not specified
        fog_color = hex_to_int("#c0d8ff")

    # Grass and foliage colours (optional, only when biome overrides default)
    grass_color = effects.get("grass_color")
    if grass_color:
        grass_color = hex_to_int(grass_color)

    foliage_color = effects.get("foliage_color")
    if foliage_color:
        foliage_color = hex_to_int(foliage_color)

    grass_modifier = effects.get("grass_color_modifier")

    return {
        "name": name,
        "temperature": temperature,
        "downfall": downfall,
        "precipitation": determine_precipitation(temperature, has_precip, name),
        "water_color": water_color,
        "sky_color": sky_color,
        "fog_color": fog_color,
        "grass_color": grass_color,
        "foliage_color": foliage_color,
        "grass_color_modifier": grass_modifier,
    }


def main():
    output_dir = os.path.join(os.path.dirname(__file__), "..", "crates", "biome-db", "data")
    os.makedirs(output_dir, exist_ok=True)

    biomes = []
    for name in BIOME_NAMES:
        url = f"{BASE_URL}/{name}.json"
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
            resp = urllib.request.urlopen(req, timeout=15)
            data = json.loads(resp.read().decode("utf-8"))
            entry = parse_biome(name, data)
            biomes.append(entry)
            print(f"  OK  {name}: temp={entry['temperature']}, down={entry['downfall']}, precip={entry['precipitation']}")
        except Exception as e:
            print(f"  FAIL {name}: {e}")

    print(f"\nTotal biomes scraped: {len(biomes)}")

    output = os.path.join(output_dir, "biomes.json")
    with open(output, "w", encoding="utf-8") as f:
        json.dump(biomes, f, indent=2)

    print(f"Written to {output}")

    # Verify with biome count
    print(f"\nBiomes with custom grass_color: {sum(1 for b in biomes if b['grass_color'] is not None)}")
    print(f"Biomes with custom foliage_color: {sum(1 for b in biomes if b['foliage_color'] is not None)}")
    print(f"Precipitation types: {set(b['precipitation'] for b in biomes)}")


if __name__ == "__main__":
    main()
