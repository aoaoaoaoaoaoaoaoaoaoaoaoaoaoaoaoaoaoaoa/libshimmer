#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["fonttools", "pillow>=11"]
# ///
"""Renders the README's roster reel, media/roster.webp, from this checkout: every stock recipe paints
its own name through the preview script, drawn as a terminal draws it.

usage: media/roster.py                       render roster.webp and record its frames' fingerprint
       media/roster.py check                 fail unless roster.webp shows this checkout's frames
       media/roster.py stills OUT.png N...   a sheet of frames N..., for review

Rendering needs ffmpeg with libwebp and the JetBrainsMono NL Nerd Font; checking needs only cargo.
"""
import functools, hashlib, json, math, subprocess, sys
from pathlib import Path

from fontTools.ttLib import TTFont
from PIL import Image, ImageChops, ImageDraw, ImageEnhance, ImageFilter, ImageFont

HERE = Path(__file__).resolve().parent
WEBP, FINGERPRINT = HERE / 'roster.webp', HERE / 'roster.sha256'
FAMILY = 'JetBrainsMono NL Nerd Font'
SIZE = 32                     # font px; the reel is drawn at twice its display size
COLUMNS, ITEM_CELLS = 3, 22   # columns of the wall, cells each item keeps
FADE_CELLS = 6                # cells over which a clipped backdrop fades out
RADIUS = 28                   # px, the card's corners
PAD, GUTTER = 72, 76          # px
FG, BG = (208, 208, 208), (14, 14, 17)
GLOW, GLOW_RADIUS = 0.55, 9
FPS = 31.25                   # one frame a 32 ms tick


def dump():
    """The reel's JSON lines from this checkout's code, and their running fingerprint."""
    digest = hashlib.sha256()
    reel = subprocess.Popen(['cargo', 'run', '-q', '--release', '--manifest-path', HERE / 'reel' / 'Cargo.toml', '--', '1'], stdout=subprocess.PIPE)

    def lines():
        for line in reel.stdout:
            digest.update(line)
            yield json.loads(line)
        if reel.wait():
            raise SystemExit(f'roster.py: the reel failed with {reel.returncode}')

    return lines(), digest


def rgb(hexcode, default):
    return default if hexcode == '-' else tuple(int(hexcode[i:i + 2], 16) for i in (0, 2, 4))


@functools.cache
def faces():
    """The primary regular and bold files, as fontconfig finds them."""
    found = []
    for style in ('Regular', 'Bold'):
        path = subprocess.run(['fc-match', '-f', '%{file}', f'{FAMILY}:style={style}'], capture_output=True, text=True, check=True).stdout
        if 'JetBrainsMono' not in Path(path).name:
            raise SystemExit(f'roster.py: install {FAMILY}; fontconfig offers {path}')
        found.append(path)
    return tuple(found)


@functools.cache
def face(path, index=0):
    return ImageFont.truetype(path, SIZE, index=index)


@functools.cache
def coverage(path, index=0):
    return set(TTFont(path, fontNumber=index, lazy=True)['cmap'].getBestCmap())


@functools.cache
def font_for(sym, heavy):
    """The font a terminal draws sym in: the primary face when it has every code point, else the
    first face fontconfig falls back to that does. Never a face that would draw a missing glyph."""
    primary = faces()[heavy]
    if all(ord(c) in coverage(primary) for c in sym):
        return face(primary)
    family = FAMILY + (':bold' if heavy else '')
    charset = ' '.join(f'{ord(c):x}' for c in sym)
    found = subprocess.run(['fc-match', '-s', '-f', '%{file}\t%{index}\n', f'{family}:charset={charset}'], capture_output=True, text=True).stdout
    for line in found.splitlines():
        path, _, index = line.partition('\t')
        if path.endswith(('.ttf', '.otf', '.ttc')) and all(ord(c) in coverage(path, int(index or 0)) for c in sym):
            return face(path, int(index or 0))
    raise SystemExit(f'roster.py: no installed font draws {sym!r}')


class Wall:
    """The roster on a card: columns of recipes, each painting its own name."""

    def __init__(self, head):
        regular = face(faces()[0])
        self.ascent, descent = regular.getmetrics()
        self.cw, self.ch = round(regular.getlength('M')), self.ascent + descent
        self.gap = round(self.ch * 0.62)
        self.octants = head['octants']
        self.rows = math.ceil(len(head['names']) / COLUMNS)
        self.width = 2 * PAD + COLUMNS * ITEM_CELLS * self.cw + (COLUMNS - 1) * GUTTER
        self.height = 2 * PAD + self.rows * self.ch + (self.rows - 1) * self.gap
        # 255 at the left edge, 0 at the right
        self.fade = Image.linear_gradient('L').rotate(90, expand=True).resize((FADE_CELLS * self.cw, self.ch))
        self.card = Image.new('L', (self.width, self.height), 0)
        ImageDraw.Draw(self.card).rounded_rectangle([0, 0, self.width - 1, self.height - 1], RADIUS, fill=255)

    def origin(self, i):
        col, row = divmod(i, self.rows)
        return PAD + col * (ITEM_CELLS * self.cw + GUTTER), PAD + row * (self.ch + self.gap)

    def block(self, draw, ch, x, y, fg, bg):
        """Draws block elements and octants on the cell grid, as terminals do natively. False if ch is not one."""
        cw, height = self.cw, self.ch
        if ch in self.octants:
            bits = self.octants[ch]
            for py in range(4):
                for px in range(2):
                    if bits >> (2 * py + px) & 1:
                        x0, x1 = x + round(px * cw / 2), x + round((px + 1) * cw / 2)
                        y0, y1 = y + round(py * height / 4), y + round((py + 1) * height / 4)
                        draw.rectangle([x0, y0, x1 - 1, y1 - 1], fill=fg)
            return True
        o = ord(ch)
        if 0x2581 <= o <= 0x2588:   # lower eighths
            h = round(height * (o - 0x2580) / 8)
            draw.rectangle([x, y + height - h, x + cw - 1, y + height - 1], fill=fg)
        elif 0x2589 <= o <= 0x258F:   # left eighths
            w = round(cw * (0x2590 - o) / 8)
            draw.rectangle([x, y, x + w - 1, y + height - 1], fill=fg)
        elif o == 0x2594:   # upper eighth
            draw.rectangle([x, y, x + cw - 1, y + round(height / 8) - 1], fill=fg)
        elif o == 0x2595:   # right eighth
            draw.rectangle([x + cw - round(cw / 8), y, x + cw - 1, y + height - 1], fill=fg)
        elif 0x2591 <= o <= 0x2593:   # shades
            a = (o - 0x2590) / 4
            draw.rectangle([x, y, x + cw - 1, y + height - 1], fill=tuple(round(f * a + b * (1 - a)) for f, b in zip(fg, bg)))
        else:
            return False
        return True

    def frame(self, frame):
        img = Image.new('RGB', (self.width, self.height), BG)
        draw = ImageDraw.Draw(img)
        for i, row in enumerate(frame['rows']):
            x0, y = self.origin(i)
            for col, (sym, fg, bg, heavy, under) in enumerate(row[:ITEM_CELLS]):
                x = x0 + col * self.cw
                fgc, bgc = rgb(fg, FG), rgb(bg, BG)
                if bgc != BG:
                    draw.rectangle([x, y, x + self.cw - 1, y + self.ch - 1], fill=bgc)
                if sym and sym != ' ' and not self.block(draw, sym[0], x, y, fgc, bgc):
                    font = font_for(sym, heavy)
                    w = font.getlength(sym)
                    draw.text((x + (self.cw * max(1, round(w / self.cw)) - w) / 2, y + self.ascent), sym, font=font, fill=fgc, anchor='ls')
                if under != '-':
                    uy = y + self.ascent + max(2, round(SIZE * 0.1))
                    draw.rectangle([x, uy, x + self.cw - 1, uy + max(1, round(SIZE / 16)) - 1], fill=rgb(under, FG))
            # A backdrop clipped at the item's edge fades out rather than stopping.
            fx = x0 + (ITEM_CELLS - FADE_CELLS) * self.cw
            box = (fx, y, fx + FADE_CELLS * self.cw, y + self.ch)
            img.paste(Image.composite(img.crop(box), Image.new('RGB', self.fade.size, BG), self.fade), box[:2])
        if GLOW:
            halo = ImageEnhance.Brightness(img.filter(ImageFilter.GaussianBlur(GLOW_RADIUS))).enhance(GLOW)
            img = ImageChops.screen(img, halo)
        img.putalpha(self.card)
        return img


def render():
    frames, digest = dump()
    wall = Wall(next(frames))
    staged = WEBP.with_name('roster.webp.part')
    encoder = subprocess.Popen(['ffmpeg', '-loglevel', 'error', '-y', '-f', 'rawvideo', '-pix_fmt', 'rgba', '-s', f'{wall.width}x{wall.height}', '-framerate', str(FPS), '-i', '-', '-c:v', 'libwebp_anim', '-pix_fmt', 'yuva420p', '-quality', '80', '-compression_level', '6', '-loop', '0', '-f', 'webp', staged], stdin=subprocess.PIPE)
    try:
        for frame in frames:
            encoder.stdin.write(wall.frame(frame).tobytes())
        encoder.stdin.close()
        if encoder.wait():
            raise SystemExit(f'roster.py: ffmpeg failed with {encoder.returncode}')
        staged.replace(WEBP)
    finally:
        staged.unlink(missing_ok=True)
    FINGERPRINT.write_text(digest.hexdigest() + '\n')


def check():
    frames, digest = dump()
    for _ in frames:
        pass
    if not FINGERPRINT.exists() or FINGERPRINT.read_text().strip() != digest.hexdigest():
        raise SystemExit('roster.py: media/roster.webp no longer shows this checkout; run media/roster.py and commit the reel')


def stills(out, picks):
    frames, _ = dump()
    wall = Wall(next(frames))
    kept = {n: frame for n, frame in enumerate(frames) if n in picks}
    sheet = Image.new('RGBA', (wall.width, wall.height * len(picks)), (255, 255, 255, 255))
    for k, n in enumerate(picks):
        sheet.alpha_composite(wall.frame(kept[n]), (0, k * wall.height))
    sheet.save(out)


def main():
    match sys.argv[1:]:
        case []:
            render()
        case ['check']:
            check()
        case ['stills', out, *picks] if picks:
            stills(out, [int(n) for n in picks])
        case _:
            raise SystemExit(__doc__)


if __name__ == '__main__':
    main()
