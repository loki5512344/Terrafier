# Build release binaries
cargo build --release

# Create dist directory
New-Item -ItemType Directory -Force -Path dist

# Copy binaries
Copy-Item target/release/terrafier-cli.exe dist/
Copy-Item target/release/terrafier-gui.exe dist/

Write-Host "Release binaries in dist/"
