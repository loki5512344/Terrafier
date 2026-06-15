"""Download Minecraft colormap PNG files for embedding."""
import urllib.request
import os

base_url = "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.11/assets/minecraft/textures/colormap"
output_dir = os.path.join(os.path.dirname(__file__), "..", "crates", "biome-db", "data")

files = ["grass.png", "foliage.png", "dry_foliage.png"]

for name in files:
    url = f"{base_url}/{name}"
    outpath = os.path.join(output_dir, name)
    print(f"Downloading {name}...")
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    resp = urllib.request.urlopen(req, timeout=15)
    data = resp.read()
    with open(outpath, "wb") as f:
        f.write(data)
    print(f"  {len(data)} bytes -> {outpath}")
