use crate::game::world::chunk::CHUNK_DIM;

// Used for tile masks
//
// pub struct ChunkTileSet(pub [[Vec<(TileType, u8)>; CHUNK_DIM]; CHUNK_DIM]);

pub type ChunkTileSet = [[Option<u8>; CHUNK_DIM]; CHUNK_DIM];

const MASK_TO_TILE: [(u8, u8); 47] = [
    (0, 0),
    (1, 1),
    (4, 2),
    (16, 3),
    (64, 4),
    (5, 5),
    (20, 6),
    (80, 7),
    (65, 8),
    (7, 9),
    (28, 10),
    (112, 11),
    (193, 12),
    (17, 13),
    (68, 14),
    (21, 15),
    (84, 16),
    (81, 17),
    (69, 18),
    (23, 19),
    (92, 20),
    (113, 21),
    (197, 22),
    (29, 23),
    (116, 24),
    (209, 25),
    (71, 26),
    (31, 27),
    (124, 28),
    (241, 29),
    (199, 30),
    (85, 31),
    (87, 32),
    (93, 33),
    (117, 34),
    (213, 35),
    (95, 36),
    (125, 37),
    (245, 38),
    (215, 39),
    (119, 40),
    (221, 41),
    (127, 42),
    (253, 43),
    (247, 44),
    (223, 45),
    (255, 46)
];

const TILEMAP_WIDTH: f32 = 11.0;
const TILEMAP_HEIGHT: f32 = 5.0;
pub const UV_LOOKUP: [[f32; 4]; 47] = [
    [3.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [10.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 11.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [10.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 11.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
];

pub fn generate_tileset_lookup() -> [u8; 256] {
    let mut tile_lookup: [u8; 256] = [0; 256];

    for i in 0..255 {
        let effective_mask = get_effective_mask(i);
        for (mask, tile) in MASK_TO_TILE {
            if effective_mask == mask {
                tile_lookup[i as usize] = tile;
            }
        }
    }

    tile_lookup
}

fn get_effective_mask(mask: u8) -> u8 {
    let n  = mask & 1 != 0;
    let ne = mask & 2 != 0;
    let e  = mask & 4 != 0;
    let se = mask & 8 != 0;
    let s  = mask & 16 != 0;
    let sw = mask & 32 != 0;
    let w  = mask & 64 != 0;
    let nw = mask & 128 != 0;

    let effective = (n  as u8)
        | ((n && e && ne) as u8) << 1
        | (e  as u8) << 2
        | ((e && s && se) as u8) << 3
        | (s  as u8) << 4
        | ((s && w && sw) as u8) << 5
        | (w  as u8) << 6
        | ((n && w && nw) as u8) << 7;

    effective
}