"""Generate the board, piece and overlay SVGs from the templates in assets/templates.

Each template has slots in double braces. Most are colours and strengths from a style's
palette below. A few hold markup this script draws itself, because a hand-written
template can't vary it: every board square cut from its own piece of wood or stone,
marble veins, the growth rings of a wooden piece, the flutes around a piece's edge.
The randomness is seeded by the style's name, so every run writes the same files.
Running this script writes complete SVGs that the app loads directly:

    assets/boards/<style>.svg
    assets/pieces/<style>/<side>_<kind>.svg      side: dark, light   kind: man, king
    assets/overlays/<name>.svg
    assets/icons/<name>.svg
    assets/ui/logo_<mode>.svg, assets/ui/background_<mode>.svg   mode: dark, light

To add or tweak a style, edit its palette here and run:  python3 scripts/generate_assets.py
"""
import colorsys
import math
import random
import re
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "assets"
TEMPLATES = ASSETS / "templates"


# Colour helpers. Shading moves the hue a little as well as the lightness: highlights
# drift warm, toward gold, and shadows drift cool, the way a painter mixes them

def clamp(value: float) -> float:
    return max(0.0, min(1.0, value))


def to_hex(rgb) -> str:
    return "#" + "".join(f"{round(clamp(c) * 255):02x}" for c in rgb)


def from_hex(color: str):
    return tuple(int(color[i:i + 2], 16) / 255 for i in (1, 3, 5))


def shade(color: str, lightness: float, saturation: float = 0.0, warmth: float = 8.0) -> str:
    """Lighten (positive) or darken (negative) a colour, turning its hue up to `warmth`
    degrees toward gold when lightening and toward blue when darkening."""
    h, l, s = colorsys.rgb_to_hls(*from_hex(color))
    target = 45 / 360 if lightness > 0 else 230 / 360
    gap = (target - h + 0.5) % 1 - 0.5
    turn = math.copysign(min(abs(gap), warmth / 360 * min(1.0, abs(lightness) * 4)), gap)
    return to_hex(colorsys.hls_to_rgb((h + turn) % 1, clamp(l + lightness), clamp(s + saturation)))


def jitter(rng: random.Random, color: str, lightness: float, hue: float, saturation: float) -> str:
    """A colour nudged at random, so neighbouring pieces of one material never match exactly."""
    h, l, s = colorsys.rgb_to_hls(*from_hex(color))
    return to_hex(colorsys.hls_to_rgb((h + rng.uniform(-hue, hue) / 360) % 1,
                                      clamp(l + rng.uniform(-lightness, lightness)),
                                      clamp(s + rng.uniform(-saturation, saturation))))


def num(value: float) -> str:
    """A coordinate with one decimal, without a trailing .0, to keep the files small."""
    text = f"{value:.1f}"
    return text[:-2] if text.endswith(".0") else text


# Shapes drawn in code

def vein(rng: random.Random, x: float, y: float, angle: float, length: float, width: float) -> tuple[str, list]:
    """A marble vein: a wandering line that swells and thins along its length and
    tapers at both ends, as a closed outline. Returns the path data and its centre line."""
    step = 4.0
    count = max(2, int(length / step))
    points, turn = [], 0.0
    for _ in range(count + 1):
        points.append((x, y))
        turn = turn * 0.6 + rng.gauss(0, 0.05)
        angle += turn
        x += step * math.cos(angle)
        y += step * math.sin(angle)

    phase, waves = rng.uniform(0, math.tau), rng.uniform(1.5, 4)
    left, right = [], []
    for i, (px, py) in enumerate(points):
        ax, ay = points[max(i - 1, 0)]
        bx, by = points[min(i + 1, count)]
        dx, dy = bx - ax, by - ay
        norm = math.hypot(dx, dy) or 1
        t = i / count
        half = width / 2 * math.sin(math.pi * t) ** 0.7 * (0.55 + 0.45 * math.sin(phase + waves * math.tau * t))
        left.append((px - dy / norm * half, py + dx / norm * half))
        right.append((px + dy / norm * half, py - dx / norm * half))
    outline = left + right[::-1]
    data = "M" + " L".join(f"{num(px)} {num(py)}" for px, py in outline) + "Z"
    return data, points


def veins(rng: random.Random, x: float, y: float, size: float, direction: float, density: float, width: float) -> str:
    """Path data for the veins of one slab of marble: a few main veins near one
    direction, branches splitting off them, and hairlines."""
    paths = []
    mains = int(density) + (rng.random() < density % 1)
    for _ in range(mains):
        angle = direction + rng.gauss(0, 0.35)
        cx, cy = x + rng.uniform(0, size), y + rng.uniform(0, size)
        length = size * rng.uniform(1.6, 2.2)
        sx, sy = cx - math.cos(angle) * length / 2, cy - math.sin(angle) * length / 2
        data, line = vein(rng, sx, sy, angle, length, width * rng.uniform(0.6, 1.4))
        paths.append(data)
        for _ in range(rng.choice((0, 1, 1, 2))):
            bx, by = line[rng.randrange(len(line) // 5, len(line) * 4 // 5)]
            branch = angle + rng.choice((-1, 1)) * rng.uniform(0.4, 0.9)
            paths.append(vein(rng, bx, by, branch, length * rng.uniform(0.2, 0.45), width * 0.5)[0])
    for _ in range(rng.choice((0, 1, 2))):
        angle = direction + rng.gauss(0, 0.6)
        hx, hy = x + rng.uniform(0, size), y + rng.uniform(0, size)
        paths.append(vein(rng, hx - math.cos(angle) * size / 2, hy - math.sin(angle) * size / 2,
                          angle, size * rng.uniform(0.6, 1.2), width * 0.3)[0])
    return "".join(paths)


# Boards: an 8x8 grid of 90px squares inside a 40px frame, light square in the top-left

SQUARE, BORDER = 90, 40


def squares(style: str, values: dict, draw) -> str:
    """Markup for all 64 squares, each a separate piece of material drawn by `draw`
    inside its own clipped viewport."""
    rng = random.Random(style)
    out = []
    for row in range(8):
        for col in range(8):
            side = "light" if (row + col) % 2 == 0 else "dark"
            body = draw(rng, side, values, row, col)
            out.append(f'<svg x="{BORDER + col * SQUARE}" y="{BORDER + row * SQUARE}" width="{SQUARE}" height="{SQUARE}">{body}</svg>')
    return "\n    ".join(out)


def wood_square(rng, side, values, row, col) -> str:
    """One square of wood: its own tone, and grain taken from a random part of the
    grain texture."""
    color = jitter(rng, values[side], 0.025, 3, 0.04)
    ox, oy = rng.randrange(0, 4000, 10), rng.randrange(0, 4000, 10)
    strength = float(values[f"{side}_grain"]) * rng.uniform(0.85, 1.1)
    return (f'<rect width="90" height="90" fill="{color}"/>'
            f'<rect x="{ox}" y="{oy}" width="90" height="90" filter="url(#grain-{side})" '
            f'transform="translate({-ox} {-oy})" opacity="{strength:.2f}"/>')


def stone_square(rng, side, values, row, col) -> str:
    """One slab of stone: its own tone, a cloudy wash, then veins with a soft halo."""
    color = jitter(rng, values[side], 0.02, 2, 0.02)
    ox, oy = rng.randrange(0, 4000, 10), rng.randrange(0, 4000, 10)
    body = (f'<rect width="90" height="90" fill="{color}"/>'
            f'<rect x="{ox}" y="{oy}" width="90" height="90" filter="url(#cloud-{side})" '
            f'transform="translate({-ox} {-oy})" opacity="{rng.uniform(0.7, 1):.2f}"/>')
    density = float(values[f"{side}_veins"])
    direction = float(values["vein_direction"]) + (0 if side == "light" else 0.5)
    data = veins(rng, 0, 0, 90, direction, density, float(values["vein_width"])) if density else ""
    if data:
        vein_id = f"vein-{row}-{col}"
        body += (f'<defs><path id="{vein_id}" d="{data}"/></defs>'
                 f'<use href="#{vein_id}" fill="{values[side + "_vein"]}" filter="url(#halo)" opacity="0.6"/>'
                 f'<use href="#{vein_id}" fill="{values[side + "_vein"]}" opacity="0.7"/>')
    return body


def wood_board(style: str, palette: dict) -> dict:
    values = {**palette}
    for side in ("light", "dark"):
        values.setdefault(f"{side}_grain_color", shade(palette[side], -0.28, 0.05))
    values.update({
        "frame_light": shade(palette["frame"], 0.06), "frame_dark": shade(palette["frame"], -0.1),
        "frame_grain_color": shade(palette["frame"], -0.22),
        "inlay_dark": shade(palette["inlay"], -0.45),
    })
    values["squares"] = squares(style, values, wood_square)
    return values


def stone_board(style: str, palette: dict) -> dict:
    values = {**palette}
    values.update({
        "frame_light": shade(palette["frame"], 0.06), "frame_dark": shade(palette["frame"], -0.1),
        "inlay_dark": shade(palette["inlay"], -0.45),
    })
    values["squares"] = squares(style, values, stone_square)
    return values


def flat_board(style: str, palette: dict) -> dict:
    return palette


BOARD_TEMPLATES = {"wood.svg": wood_board, "stone.svg": stone_board, "flat.svg": flat_board}

# Wood: the colour of each kind of square, how strongly its grain shows, and how much
# figure (the arches of growth rings) the grain has. Inlay is the thin stringing line
# around the playing field
BOARDS = {
    "walnut": ("wood.svg", {
        "light": "#dcbc8a", "dark": "#5e3a22",
        "light_grain": "0.55", "dark_grain": "0.7",
        "light_figure": "0.55", "dark_figure": "0.45",
        "frame": "#4a2c18", "inlay": "#e9d6ae", "sheen": "1",
    }),
    "mahogany": ("wood.svg", {
        "light": "#e8c9a0", "dark": "#80321b",
        "light_grain": "0.5", "dark_grain": "0.65",
        "light_figure": "0.5", "dark_figure": "0.6",
        "frame": "#5a2212", "inlay": "#d9b26a", "sheen": "1",
    }),
    "ebony": ("wood.svg", {
        "light": "#e4d8bf", "dark": "#2a221d",
        "light_grain": "0.4", "dark_grain": "0.8",
        "light_figure": "0.35", "dark_figure": "0.3",
        "dark_grain_color": "#0a0705",
        "frame": "#231c18", "inlay": "#c9a45c", "sheen": "1.2",
    }),
    # Light white oak against fumed oak: pale, matte, modern
    "oak": ("wood.svg", {
        "light": "#e7d6b6", "dark": "#7a6650",
        "light_grain": "0.5", "dark_grain": "0.55",
        "light_figure": "0.7", "dark_figure": "0.7",
        "frame": "#5d4d3c", "inlay": "#2f261d", "sheen": "0.5",
    }),
    # A painted wooden board: red and black paint with the grain showing through
    "classic": ("wood.svg", {
        "light": "#a8291f", "dark": "#1f1d1c",
        "light_grain": "0.22", "dark_grain": "0.35",
        "light_figure": "0.4", "dark_figure": "0.4",
        "light_grain_color": "#5c120c", "dark_grain_color": "#000000",
        "frame": "#2a2522", "inlay": "#c9a45c", "sheen": "1.3",
    }),
    "tournament": ("flat.svg", {
        "light": "#ece3c4", "dark": "#3f7046",
        "frame_top": "#2c4f31", "frame_bottom": "#152a19",
        "mottle": "0.06", "texture": "0.08",
    }),
    # Stone: veins per slab (fractions mean some slabs get one more), their colour and
    # width, and the colour of the cloudy wash under them
    "marble": ("stone.svg", {
        "light": "#ece9e3", "dark": "#2c4a3e",
        "light_vein": "#9a9ca3", "dark_vein": "#d8e6dd",
        "light_cloud": "#b9bbc0", "dark_cloud": "#0f2a20",
        "light_veins": "2.4", "dark_veins": "1.8", "vein_width": "2.2", "vein_direction": "0.6",
        "frame": "#2f2b29", "inlay": "#c9a45c",
    }),
    "onyx": ("stone.svg", {
        "light": "#f1ebdf", "dark": "#1b1a1d",
        "light_vein": "#b38a3e", "dark_vein": "#e4e0d8",
        "light_cloud": "#d8ccb4", "dark_cloud": "#3a383d",
        "light_veins": "1.8", "dark_veins": "1.8", "vein_width": "1.8", "vein_direction": "-0.7",
        "frame": "#232226", "inlay": "#c9a45c",
    }),
    # Pale limestone against blue-black slate: few veins, more cloud
    "slate": ("stone.svg", {
        "light": "#cfcbc2", "dark": "#3a4250",
        "light_vein": "#9c968b", "dark_vein": "#59647a",
        "light_cloud": "#a29c90", "dark_cloud": "#232a36",
        "light_veins": "0.3", "dark_veins": "0.4", "vein_width": "1.0", "vein_direction": "0.1",
        "frame": "#2a303a", "inlay": "#9aa3b2",
    }),
}


# Pieces: a disc seen from slightly above. The face is centred on (50, 47) with radii
# 39 x 37.5, and its edge is 7 deep. Kings reuse the same disc, moved up

FACE = (50, 47, 39, 37.5)
DEPTH = 7


def flutes() -> str:
    """The ridges milled around a piece's edge: a dark line and a light line for each,
    closer together toward the sides where the edge turns away."""
    cx, cy, rx, ry = FACE
    dark, light = [], []
    for i in range(1, 44):
        t = math.pi * i / 44
        x, y = cx + rx * math.cos(t), cy + ry * math.sin(t)
        dark.append(f"M{num(x)} {num(y)}v{DEPTH}")
        light.append(f"M{num(x + 0.7 * math.sin(t))} {num(y)}v{DEPTH}")
    return (f'<path d="{"".join(dark)}" stroke="#000" stroke-opacity="0.4" stroke-width="0.7"/>'
            f'<path d="{"".join(light)}" stroke="#fff" stroke-opacity="0.16" stroke-width="0.6"/>')


def rings(rng: random.Random, color: str) -> str:
    """End grain for a piece cut from a dowel: growth rings around a pith a little off
    centre, wobbling together, with a couple of small radial cracks."""
    cx, cy, rx, ry = FACE
    px, py = cx + rng.uniform(-12, 12), cy + rng.uniform(-10, 10)
    wobble = [(rng.uniform(0.03, 0.06), k, rng.uniform(0, math.tau)) for k in (2, 3, 5)]
    out, radius = [], rng.uniform(1.5, 3)
    while radius < 80:
        points = []
        for i in range(72):
            t = math.tau * i / 72
            scale = 1 + sum(a * math.sin(k * t + p) for a, k, p in wobble) * min(1, radius / 12)
            points.append((px + radius * scale * math.cos(t), py + radius * scale * math.sin(t) * ry / rx))
        data = "M" + " L".join(f"{num(x)} {num(y)}" for x, y in points) + "Z"
        out.append(f'<path d="{data}" stroke-width="{rng.uniform(0.4, 1.3):.2f}" stroke-opacity="{rng.uniform(0.25, 0.6):.2f}"/>')
        radius += rng.uniform(2.0, 4.2)
    for _ in range(rng.choice((1, 2, 2, 3))):
        a = rng.uniform(0, math.tau)
        r1, r2 = rng.uniform(1, 4), rng.uniform(7, 16)
        out.append(f'<path d="M{num(px + r1 * math.cos(a))} {num(py + r1 * math.sin(a))} '
                   f'L{num(px + r2 * math.cos(a))} {num(py + r2 * math.sin(a))}" stroke-width="0.5" stroke-opacity="0.6"/>')
    return f'<g fill="none" stroke="{color}">{"".join(out)}</g>'


def lattice() -> str:
    """Path data for the backdrop's lattice: a checkerboard of 80px squares turned 45
    degrees about the middle of the 1920x1080 screen, keeping only the dark squares."""
    half = 80 / math.sqrt(2)
    out = []
    for i in range(-16, 16):
        for j in range(-16, 16):
            if (i + j) % 2:
                continue
            u, v = i * 80 + 40, j * 80 + 40
            x, y = 960 + (u - v) / math.sqrt(2), 540 + (u + v) / math.sqrt(2)
            if -half < x < 1920 + half and -half < y < 1080 + half:
                out.append(f"M{num(x)} {num(y - half)}L{num(x + half)} {num(y)}L{num(x)} {num(y + half)}L{num(x - half)} {num(y)}Z")
    return "".join(out)


def surface(rng: random.Random, kind: str, values: dict) -> str:
    """The material showing on a piece's face, clipped to it by the template."""
    texture = f'<rect width="100" height="100" filter="url(#speckle)" opacity="{values["speckle"]}"/>'
    if kind == "rings":
        return rings(rng, values["grain_color"]) + texture
    if kind == "veins":
        data = veins(rng, 6, 4, 88, float(values["vein_direction"]), float(values["veins"]), float(values["vein_width"]))
        return (f'<rect width="100" height="100" filter="url(#cloud)" opacity="0.8"/>{texture}'
                f'<defs><path id="veins" d="{data}"/></defs>'
                f'<use href="#veins" fill="{values["vein_color"]}" filter="url(#halo)" opacity="0.5"/>'
                f'<use href="#veins" fill="{values["vein_color"]}" opacity="0.85"/>')
    if kind == "cloud":
        return f'<rect width="100" height="100" filter="url(#cloud)"/>{texture}'
    if kind == "flecks":
        return (f'<rect width="100" height="100" filter="url(#cloud)" opacity="0.6"/>{texture}'
                f'<rect width="100" height="100" filter="url(#flecks)"/>')
    return texture


GOLD = {"inlay": "#e2bd62"}
PIECE_DEFAULTS = {
    "gloss": "0.6", "groove": "1", "flute_strength": "1", "lathe": "0",
    "speckle": "0.10", "speckle_color": "#000000", "speckle_frequency": "0.9",
    "cloud_color": "#ffffff", "fleck_color": "#ffffff", "glow": "0",
    "grain_color": "#000000", "vein_color": "#000000", "veins": "0", "vein_width": "1", "vein_direction": "0.6",
}


def piece_values(palette: dict) -> dict:
    """A piece palette's slots: its face colour shaded into a lit ramp for the face,
    the edge and the crown's inlay, plus everything left at its default."""
    values = {**PIECE_DEFAULTS, **palette}
    base = palette["face"]
    values.update({
        "face_light": shade(base, 0.14), "face_dark": shade(base, -0.16),
        "edge_light": shade(base, 0.02), "edge": shade(base, -0.1), "edge_dark": shade(base, -0.28),
        "glow_color": shade(base, 0.22, 0.15),
        "inlay_light": shade(palette["inlay"], 0.2), "inlay_dark": shade(palette["inlay"], -0.25),
    })
    return values


# Piece styles, one palette per side. `surface` picks what shows on the face: speckle
# (fine lacquer texture), rings (end grain), veins (marble), cloud (jade) or flecks
# (lapis). Glow lights the side away from the light, for stones light passes through
PIECES = {
    "classic": {
        "dark": {**GOLD, "face": "#2b2826", "surface": "speckle", "gloss": "0.55"},
        "light": {**GOLD, "face": "#b8271d", "surface": "speckle", "gloss": "0.6", "groove": "0.7"},
    },
    "ivory": {
        "dark": {**GOLD, "face": "#2a1f1a", "surface": "speckle", "gloss": "0.5", "speckle": "0.12"},
        "light": {"face": "#ede2c8", "inlay": "#b88a3a", "surface": "speckle", "gloss": "0.55",
                  "groove": "0.55", "speckle": "0.05"},
    },
    "wood": {
        "dark": {"face": "#5b3720", "inlay": "#e6c47a", "surface": "rings", "grain_color": "#24120a",
                 "gloss": "0.25", "flute_strength": "0", "lathe": "1", "speckle": "0.12", "groove": "0.8"},
        "light": {"face": "#dcb57e", "inlay": "#5a3a1e", "surface": "rings", "grain_color": "#9b6b3c",
                  "gloss": "0.25", "flute_strength": "0", "lathe": "0.7", "speckle": "0.08", "groove": "0.6"},
    },
    "marble": {
        "dark": {"face": "#1f1e21", "inlay": "#e4e0d8", "surface": "veins", "vein_color": "#e1ddd5",
                 "cloud_color": "#3b393f", "veins": "1.2", "vein_width": "1.1", "gloss": "0.6",
                 "flute_strength": "0", "speckle": "0.05"},
        "light": {"face": "#ece8e1", "inlay": "#a88442", "surface": "veins", "vein_color": "#8a8a90",
                  "cloud_color": "#c4c3c4", "veins": "1.5", "vein_width": "1.1", "gloss": "0.6",
                  "flute_strength": "0", "groove": "0.5", "speckle": "0.04"},
    },
    # Honey amber that glows where light passes through, against lapis with gold flecks
    "amber": {
        "dark": {**GOLD, "face": "#1d3f86", "surface": "flecks", "fleck_color": "#f0cf6a",
                 "cloud_color": "#6a8fd0", "gloss": "0.6", "speckle": "0.06"},
        "light": {"face": "#cf7d14", "inlay": "#6a3608", "surface": "cloud", "cloud_color": "#ffc65a",
                  "glow": "0.9", "gloss": "0.7", "groove": "0.6", "speckle": "0.03"},
    },
    # Nephrite jade against white jade, both translucent with a soft cloudy body
    "jade": {
        "dark": {"face": "#2f6b4a", "inlay": "#e8d9a6", "surface": "cloud", "cloud_color": "#7fb592",
                 "glow": "0.45", "gloss": "0.45", "flute_strength": "0", "speckle": "0.06"},
        "light": {"face": "#e4e6d6", "inlay": "#5d7d63", "surface": "cloud", "cloud_color": "#ffffff",
                  "glow": "0.35", "gloss": "0.45", "flute_strength": "0", "groove": "0.5", "speckle": "0.04"},
    },
}

# Overlays and icons are drawn in white and tinted by the app, so one file serves any colour
OVERLAYS = ["move", "capture", "selected"]
ICONS = ["undo", "redo", "new_game", "home", "settings", "back", "flip_board",
         "one_player", "two_players", "win", "draw", "loading",
         "play", "exit", "board", "pieces", "dark_mode", "light_mode", "show_moves", "hide_moves",
         "position", "warning", "repetition", "move_limit"]
TINTED = {"color": "#ffffff"}


# The logo's letters take each theme's text colour; its crown and rule are gold, deeper in
# light mode so they hold up against the grey
LOGOS = {
    "dark": {"text": "#e4e6e9", "gold_light": "#f2d27c", "gold": "#d6a646", "gold_dark": "#8f6a24", "shadow": "0.45"},
    "light": {"text": "#2a2118", "gold_light": "#d0a24a", "gold": "#a8771f", "gold_dark": "#6e4c12", "shadow": "0.18"},
}

# Backdrops: cloth under a pool of light, so the boards bring the colour. The dark one is neutral
# grey; the light one is a mid-toned warm stone, since gold looks muddy on cool greys, matching the
# light palette in src/gui/theme/palette.rs. Fibres and threads set how strongly the cloth's
# texture shows; the motif is the lattice near the edges
BACKGROUNDS = {
    "dark": {"center": "#2b2d31", "middle": "#1d1e21", "edge": "#0b0b0d",
             "fibres": "0.06", "threads": "0.04", "motif": "#ffffff", "motif_strength": "0.035",
             "vignette": "0.5"},
    "light": {"center": "#b0a797", "middle": "#a49b8c", "edge": "#857b6c",
              "fibres": "0.05", "threads": "0.06", "motif": "#000000", "motif_strength": "0.045",
              "vignette": "0.12"},
}


def fill(template_path: Path, values: dict) -> str:
    """Fill a template's slots, failing loudly if any slot is missing a value."""
    text = template_path.read_text()
    for key, value in values.items():
        text = text.replace("{{" + key + "}}", str(value))

    unfilled = re.findall(r"\{\{(\w+)\}\}", text)
    if unfilled:
        raise ValueError(f"{template_path.name}: no value for {sorted(set(unfilled))}")

    # Mark the output as generated, right after the opening <svg> tag
    source = template_path.relative_to(ASSETS.parent)
    note = f"\n  <!-- Generated from {source} by scripts/generate_assets.py. Edit the template, not this file -->"
    opening_end = text.index(">") + 1
    return text[:opening_end] + note + text[opening_end:]


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    print("wrote", path.relative_to(ASSETS.parent))


def main() -> None:
    for style, (template, palette) in BOARDS.items():
        values = BOARD_TEMPLATES[template](style, palette)
        write(ASSETS / "boards" / f"{style}.svg", fill(TEMPLATES / "boards" / template, values))

    edge = flutes()
    for style, sides in PIECES.items():
        for side, palette in sides.items():
            values = piece_values(palette)
            for kind in ("man", "king"):
                rng = random.Random(f"{style}/{side}/{kind}")
                slots = {**values, "flutes": edge, "surface": surface(rng, palette["surface"], values)}
                text = fill(TEMPLATES / "pieces" / f"{kind}.svg", slots)
                write(ASSETS / "pieces" / style / f"{side}_{kind}.svg", text)

    for name in OVERLAYS:
        write(ASSETS / "overlays" / f"{name}.svg", fill(TEMPLATES / "overlays" / f"{name}.svg", TINTED))
    for name in ICONS:
        write(ASSETS / "icons" / f"{name}.svg", fill(TEMPLATES / "icons" / f"{name}.svg", TINTED))

    for mode, values in LOGOS.items():
        write(ASSETS / "ui" / f"logo_{mode}.svg", fill(TEMPLATES / "ui" / "logo.svg", values))
    for mode, values in BACKGROUNDS.items():
        write(ASSETS / "ui" / f"background_{mode}.svg",
              fill(TEMPLATES / "ui" / "background.svg", {**values, "lattice": lattice()}))


if __name__ == "__main__":
    main()
