"""Draws the app icon (a house with a glowing door on a lawn) as a PNG,
with no dependencies, for iconutil to turn into AppIcon.icns."""
import struct, sys, zlib, math

def png(path, size):
    px = []
    for y in range(size):
        row = []
        for x in range(size):
            u, v = x / size, y / size
            r, g, b, a = 0, 0, 0, 0
            # rounded square background: sky to lawn
            cx, cy = abs(u - 0.5), abs(v - 0.5)
            k = 0.22
            dx, dy = max(cx - (0.5 - k), 0), max(cy - (0.5 - k), 0)
            if dx * dx + dy * dy <= k * k:
                a = 255
                if v < 0.62:
                    t = v / 0.62
                    r, g, b = int(150 + 90 * t), int(205 + 35 * t), int(240 - 10 * t)
                else:
                    r, g, b = 86, 160, 80
                # house body
                if 0.26 < u < 0.74 and 0.42 < v < 0.72:
                    r, g, b = 232, 240, 228
                # roof
                if 0.42 >= v >= 0.24 and abs(u - 0.5) < (v - 0.24) / 0.18 * 0.28 + 0.0:
                    r, g, b = 134, 179, 162
                # glowing door
                if 0.45 < u < 0.55 and 0.52 < v < 0.72:
                    r, g, b = 255, 214, 150
                # windows
                if (0.31 < u < 0.4 or 0.6 < u < 0.69) and 0.5 < v < 0.59:
                    r, g, b = 255, 240, 194
            row.append(bytes((r, g, b, a)))
        px.append(b"\0" + b"".join(row))
    raw = zlib.compress(b"".join(px), 9)
    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)) + chunk(b"IDAT", raw) + chunk(b"IEND", b""))

out = sys.argv[1]
for s in (16, 32, 64, 128, 256, 512, 1024):
    png(f"{out}/icon_{s}x{s}.png", s)
