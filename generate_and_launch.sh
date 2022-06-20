python3 generator/generator.py
cp scene.json scenes/generated.json
cargo run --release -- --width 600 --height 600 --output output.png --scene scenes/generated.json --per-pixel-samples=16 --shadow-samples=16 --threads=8 && open output.png

