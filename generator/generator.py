import random
import time


class Vec3:
    def __init__(self, x: float, y: float, z: float):
        self.x = x
        self.y = y
        self.z = z


class Builder:
    def __init__(self):
        self.objects = []

    def add(self, object: str):
        self.objects.append(object)

    def render(self) -> str:
        return ','.join(self.objects)


class Color:
    def __init__(self, r: float, g: float, b: float):
        if r > 1.0 or r < 0.0:
            raise "red channel must be in [0..1] range"

        if g > 1.0 or g < 0.0:
            raise "green channel must be in [0..1] range"

        if b > 1.0 or b < 0.0:
            raise "blue channel must be in [0..1] range"

        self.r = r
        self.g = g
        self.b = b


def build_camera(fov: int, pos: Vec3, lookat: Vec3):
    return """
        {{
            "type": "camera",
            "pos": {{
                "x": {posx},
                "y": {posy},
                "z": {posz}
            }},
            "lookat": {{
                "x": {lookatx},
                "y": {lookaty},
                "z": {lookatz}
            }},
            "fov": 60.0
        }}
    """.format(posx=pos.x, posy=pos.y, posz=pos.z, lookatx=lookat.x, lookaty=lookat.y, lookatz=lookat.z, fov=fov)


def build_sphere(pos: Vec3, radius: float, ttype: str, albedo: float, color: Color):
    return """
            {{
                "type": "sphere",
                "pos": {{
                    "x": {x},
                    "y": {y},
                    "z": {z}
                }},
                "radius": {radius},
                "material": {{
                    "type": "metal",
                    "fuzz": 0.0,
                    "albedo": {albedo},
                    "color": {{
                        "r": {color_r},
                        "g": {color_g},
                        "b": {color_b}
                    }}
                }}
            }}
            """.format(
        x=pos.x,
        y=pos.y,
        z=pos.z,
        radius=radius,
        albedo=albedo,
        color_r=color.r,
        color_g=color.g,
        color_b=color.b
    )


def build_light(pos: Vec3, radius: float):
    return """
            {{
                "type": "point_light",
                "pos": {{
                    "x": {x},
                    "y": {y},
                    "z": {z}
                }},
                "radius": {radius},
                "material": {{
                    "type": "lambertian",
                    "fuzz": 0.0,
                    "albedo": 1.0,
                    "color": {{
                        "r": 1.0,
                        "g": 1.0,
                        "b": 1.0
                    }}
                }}
            }}
            """.format(
        x=pos.x,
        y=pos.y,
        z=pos.z,
        radius=radius
    )


builder = Builder()
builder.add(build_camera(60, Vec3(6.0, 6.4, 2.0), Vec3(5.0, 3.0, 7.0)))

gridsize = 15
colors = [Color(1.0, 0.0, 0.0),
          Color(0.0, 1.0, 0.0),
          Color(0.0, 0.0, 1.0)
          ]

random.seed(time.time())
for i in range(0, gridsize):
    for j in range(0, gridsize):
        pos = Vec3(i, random.randint(1, 10) / 2.0, j)
        # col = Color(i / gridsize, j / gridsize, 1.0)
        col = colors[random.randint(1, len(colors))-1]

        builder.add(build_sphere(pos,
                                 0.2 + random.randint(1, 100) / 400.0, "metal", 0.8, col))

builder.add(build_light(Vec3(100, 100, 10), 40.0))
builder.add(build_sphere(Vec3(0.0, -99.0, 0.0),
                         100.0, "lambertian", 0.6, Color(0.4, 0.8, 0.1)))

with open("scene.json", "w") as file:
    file.write("[" + builder.render() + "]")
