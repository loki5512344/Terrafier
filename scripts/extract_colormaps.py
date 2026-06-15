"""Download and extract Minecraft colour map PNGs as raw RGB arrays.

The grass.png and foliage.png are 256x256 colour lookup tables.
x-axis = temperature (0-255), y-axis = downfall (0-255).
"""
import urllib.request
import struct
import os
import json
import zlib


def download_file(url):
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    resp = urllib.request.urlopen(req, timeout=15)
    return resp.read()


def parse_png_rgb(data):
    """Parse a PNG file and extract RGB pixel data as a flat array."""
    # PNG signature check
    assert data[:8] == b'\x89PNG\r\n\x1a\n', "Not a valid PNG"

    # Find IHDR and IDAT chunks
    pos = 8
    chunks = []
    idat_data = b''

    while pos < len(data):
        length = struct.unpack('>I', data[pos:pos+4])[0]
        chunk_type = data[pos+4:pos+8]
        chunk_data = data[pos+8:pos+8+length]

        if chunk_type == b'IHDR':
            width = struct.unpack('>I', chunk_data[0:4])[0]
            height = struct.unpack('>I', chunk_data[4:8])[0]
            bit_depth = chunk_data[8]
            color_type = chunk_data[9]
        elif chunk_type == b'IDAT':
            idat_data += chunk_data
        elif chunk_type == b'IEND':
            break

        pos += 12 + length

    # Decompress
    raw_data = zlib.decompress(idat_data)

    # Extract RGB pixels (filter byte per row)
    pixels = []
    row_size = 1 + width * 3  # filter byte + RGB per pixel
    for y in range(height):
        row_start = y * row_size + 1  # skip filter byte
        for x in range(width):
            offset = row_start + x * 3
            r = raw_data[offset]
            g = raw_data[offset + 1]
            b = raw_data[offset + 2]
            pixels.append([r, g, b])

    return width, height, pixels


def main():
    base_url = "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.11/assets/minecraft/textures/colormap"
    output_dir = os.path.join(os.path.dirname(__file__), "..", "crates", "biome-db", "data")

    for name in ["grass", "foliage"]:
        url = f"{base_url}/{name}.png"
        print(f"Downloading {name}.png...")
        png_data = download_file(url)
        width, height, pixels = parse_png_rgb(png_data)
        print(f"  Size: {width}x{height}, pixels: {len(pixels)}")

        # Save as JSON array: [[r,g,b], ...] flattened
        flat = [v for pixel in pixels for v in pixel]
        output = {
            "width": width,
            "height": height,
            "data": flat,
        }

        out_path = os.path.join(output_dir, f"{name}_colormap.json")
        with open(out_path, "w") as f:
            json.dump(output, f)
        print(f"  Written to {out_path}")


if __name__ == "__main__":
    main()
