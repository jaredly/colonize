use std::collections::HashMap;

use noise::{ Seed, open_simplex2 };
use cgmath::Point3;

use { CHUNK_SIZE, LOG2_OF_CHUNK_SIZE };
use chunk::Chunk;
use terrain::{ Tile, TileType };
use mapgen;

const NOISE_SCALING_FACTOR: f64 = 64.0;
const NOISE_GENERATOR: fn(&Seed, &[f64; 2]) -> f64 = scaled_open_simplex2;

pub struct Area {
    chunks: HashMap<Point3<i32>, Chunk>,
    seed: Seed,
}

impl Area {
    pub fn new(rng_seed: u32, initial_size: u32) -> Self {
        let mut area = Area {
            chunks: HashMap::new(),
            seed: Seed::new(rng_seed),
        };

        // We take a u32 and convert to an i32 internally because we generate
        // around (0, 0). but we also want to only accept valid input.
        // TODO: find a better way to do this.
        let initial_size = initial_size as i32;

        for z in -initial_size..initial_size {
            for x in -initial_size..initial_size {
                // Since the height map is 2D, along the X and Y axes, we only
                // generate it once per column.
                // This reduces the number of calls to `generate_height_map`
                // from `initial_size^3` to `initial_size^2`.
                let mut pos = Point3::new(x, 0, z);
                let height_map = mapgen::generate_height_map(
                    &area.seed,
                    &pos,
                    NOISE_GENERATOR);
                for y in -initial_size..initial_size {
                    pos.y = y;
                    mapgen::generate_chunk(
                        pos,
                        height_map,
                        |p, c| { area.add_chunk(p, c); });
                }
            }
        }

        area
    }

    pub fn add_chunk(&mut self, p: Point3<i32>, c: Chunk) {
        self.chunks.insert(p, c);
    }

    pub fn get_chunk(&self, p: Point3<i32>) -> Option<&Chunk> {
        self.chunks.get(&p)
    }

    pub fn get_tile(&self, p: &Point3<i32>) -> Tile {
        let chunk_pos = abs_pos_to_chunk_pos(p);
        let tile_pos = abs_pos_to_rel_chunk_pos(p);

        match self.get_chunk(chunk_pos) {
            Some(chunk) => chunk.tiles[tile_pos[1]][tile_pos[0]][tile_pos[2]],
            None => Tile::new(TileType::OutOfBounds),
        }
    }
}

fn scaled_open_simplex2(seed: &Seed, point: &[f64; 2]) -> f64 {
    open_simplex2(seed, &[point[0] / NOISE_SCALING_FACTOR, point[1] / NOISE_SCALING_FACTOR])
}

/// Takes an absolute coordinate and returns the origin coordinate of the chunk
/// in which this coordinate is located.
pub fn abs_pos_to_chunk_pos(p: &Point3<i32>) -> Point3<i32> {
    Point3::new(
        p[0] >> LOG2_OF_CHUNK_SIZE,
        p[1] >> LOG2_OF_CHUNK_SIZE,
        p[2] >> LOG2_OF_CHUNK_SIZE,
    )
}

/// Takes an absolute coordinate and returns the position of the coordinate
/// relative to the origin of the chunk in which the coordinate is located.
pub fn abs_pos_to_rel_chunk_pos(p: &Point3<i32>) -> Point3<usize> {
    Point3::new(
        ((p[0] % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
        ((p[1] % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
        ((p[2] % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
    )
}
