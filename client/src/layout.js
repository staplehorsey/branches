// Shared constants for the liminal-houses@1 generator. Every house sits at
// the centre of a grid cell; its interior lives in a "pocket" directly below
// it, so walking through the front door is a pure vertical translation.

export const CELL = 24;
export const HOUSE_W = 10;
export const HOUSE_D = 8;
export const WALL_H = 3.2;
export const DOOR_W = 1.3;
export const DOOR_H = 2.25;
export const ARCH_W = 2.2;
export const ARCH_H = 2.7;

const mod = (a, n) => ((a % n) + n) % n;

// 25 stacked levels so neighbouring pockets never share a height.
export function pocketY(x, z) {
  return -60 - (mod(x, 5) * 5 + mod(z, 5)) * 14;
}

export function houseCenter(x, z) {
  return { x: x * CELL, z: z * CELL };
}

export function cellOf(px, pz) {
  return { x: Math.round(px / CELL), z: Math.round(pz / CELL) };
}

export const OUTDOOR_MIN_Y = -20;
