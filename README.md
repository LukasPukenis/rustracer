# Running
It is highly adviced to run in release mode as otherwise it's very slow

```
cargo run --release -- --width 400 --height 400 --output scene1.png --scene scenes/scene_1.json --per-pixel-samples=8 --shadow-samples=10 --threads=8
```

# Parameters
- width <WIDTH>
- height <HEIGHT>
- output <OUTPUT>
- scene <SCENE>
- per-pixel-samples <PER_PIXEL_SAMPLES>
- shadow-samples <SHADOW_SAMPLES>
- threads <THREADS>

# Examples
Rendered various scenes with various parameters collected over the time showcasing the raytracer
  
![alt text](https://github.com/LukasPukenis/rustracer/blob/master/images/output-shadow64-pp4.png)
![alt text](https://github.com/LukasPukenis/rustracer/blob/master/images/output-1600-1600-16-128.png)
![alt text](https://github.com/LukasPukenis/rustracer/blob/master/images/image-600-600-pp16-shadow16.png)
