# Running
It is highly adviced to run in release mode as otherwise it's very slow

# Parameters
- width <WIDTH>
- height <HEIGHT>
- output <OUTPUT>
- scene <SCENE>
- per-pixel-samples <PER_PIXEL_SAMPLES>
- shadow-samples <SHADOW_SAMPLES>
- threads <THREADS>

# Example
```
cargo run --release -- --width 400 --height 400 --output scene1.png --scene scenes/scene_1.json --per-pixel-samples=8 --shadow-samples=10 --threads=8
```
