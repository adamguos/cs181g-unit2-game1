use std::collections::HashMap;
use std::rc::Rc;

use crate::screen::Screen;
use crate::texture::Texture;
use crate::types::{Rect, Vec2i};

pub const TILE_SZ: usize = 16;

#[derive(Clone, Copy)]
pub struct Tile {
    pub solid: bool,
}

pub struct Tileset {
    /**
     * A set of tiles used in multiple Tilemaps.
     *
     * Each tileset is a distinct image.
     */
    pub tiles: Vec<Tile>,
    texture: Rc<Texture>,
    pub tile_ids: HashMap<String, Vec<usize>>,
}

/// Indices into a Tileset
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TileID(usize);

/// Grab a tile with a given ID
impl std::ops::Index<TileID> for Tileset {
    type Output = Tile;

    fn index(&self, id: TileID) -> &Self::Output {
        &self.tiles[id.0]
    }
}

impl Tileset {
    pub fn new(
        tiles: Vec<Tile>,
        texture: &Rc<Texture>,
        tile_ids: HashMap<String, Vec<usize>>,
    ) -> Self {
        Self {
            tiles,
            texture: Rc::clone(texture),
            tile_ids,
        }
    }

    /// Get the frame rect for a tile ID
    fn get_rect(&self, id: TileID) -> Rect {
        let idx = id.0;
        let (w, _h) = self.texture.size();
        let tw = w / TILE_SZ;
        let row = idx / tw;
        let col = idx - (row * tw);

        Rect {
            x: col as i32 * TILE_SZ as i32,
            y: row as i32 * TILE_SZ as i32,
            w: TILE_SZ as u16,
            h: TILE_SZ as u16,
        }
    }

    /// Does this tileset have a title for "id"?
    fn contains(&self, id: TileID) -> bool {
        id.0 < self.tiles.len()
    }
}

#[derive(Clone)]
pub struct Tilemap {
    /// Where the tilemap is in space
    pub position: Vec2i,
    /// How big it is
    dims: (usize, usize),
    /// Which tileset is used for this tilemap
    pub tileset: Rc<Tileset>,
    /// A row-major grid of tile IDs in tileset
    map: Vec<TileID>,
}

impl Tilemap {
    /// Draws the portion of self appearing within screen.
    /// This could just as well be an extension trait on Screen defined in =tiles.rs= or
    /// something, like we did for =sprite.rs= and =draw_sprite=.
    pub fn draw(&self, screen: &mut Screen) {
        let Rect {
            x: sx,
            y: sy,
            w: sw,
            h: sh,
        } = screen.bounds();
        // We'll draw from the topmost/leftmost visible tile to the bottommost/rightmost visible tile.
        // The camera combined with out position and size tell us what's visible.
        // leftmost tile: get camera.x into our frame of reference, then divide down to tile units
        // Note that it's also forced inside of 0..self.size.0
        let left = ((sx - self.position.0) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.0 as i32) as usize;
        // rightmost tile: same deal, but with screen.x + screen.w plus a little padding to be sure we draw the rightmost tile even if it's a bit off screen.
        let right = ((sx + ((sw + TILE_SZ as u16) as i32) - self.position.0) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.0 as i32) as usize;
        // ditto top and bot
        let top = ((sy - self.position.1) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.1 as i32) as usize;
        let bot = ((sy + ((sh + TILE_SZ as u16) as i32) - self.position.1) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.1 as i32) as usize;
        // Now draw the tiles we need to draw where we need to draw them.
        // Note that we're zipping up the row index (y) with a slice of the map grid containing the necessary rows so we can avoid making a bounds check for each tile.
        for (y, row) in (top..bot)
            .zip(self.map[(top * self.dims.0)..(bot * self.dims.0)].chunks_exact(self.dims.0))
        {
            // We are in tile coordinates at this point so we'll need to translate back to pixel units and world coordinates to draw.
            let ypx = (y * TILE_SZ) as i32 + self.position.1;
            // Here we can iterate through the column index and the relevant slice of the row in parallel
            for (x, id) in (left..right).zip(row[left..right].iter()) {
                let xpx = (x * TILE_SZ) as i32 + self.position.0;
                let frame = self.tileset.get_rect(*id);
                screen.bitblt(&self.tileset.texture, frame, Vec2i(xpx, ypx));
            }
        }
    }

    pub fn new(
        position: Vec2i,
        dims: (usize, usize),
        tileset: &Rc<Tileset>,
        map: Vec<usize>,
    ) -> Self {
        assert_eq!(dims.0 * dims.1, map.len(), "Tilemap is the wrong size");
        assert!(
            map.iter().all(|tid| tileset.contains(TileID(*tid))),
            "Tilemap refers to nonexistent tiles"
        );

        Self {
            position,
            dims,
            tileset: Rc::clone(tileset),
            map: map.into_iter().map(TileID).collect(),
        }
    }

    #[allow(dead_code)]
    pub fn tile_id_at(&self, Vec2i(x, y): Vec2i) -> TileID {
        // Translate into map coordinates
        let x = (x - self.position.0) / TILE_SZ as i32;
        let y = (y - self.position.1) / TILE_SZ as i32;

        assert!(
            x >= 0 && x < self.dims.0 as i32,
            "Tile X coordinate {} out of bounds {}",
            x,
            self.dims.0
        );
        assert!(
            y >= 0 && y < self.dims.1 as i32,
            "Tile Y coordinate {} out of bounds {}",
            y,
            self.dims.1
        );
        self.map[y as usize * self.dims.0 + x as usize]
    }

    #[allow(dead_code)]
    pub fn size(&self) -> (usize, usize) {
        self.dims
    }

    #[allow(dead_code)]
    pub fn tile_at(&self, posn: Vec2i) -> Tile {
        self.tileset[self.tile_id_at(posn)]
    }

    pub fn is_visible(&self, screen_pos: Vec2i, screen_dim: Vec2i) -> bool {
        let dims_px = Vec2i(
            (self.dims.0 * TILE_SZ) as i32,
            (self.dims.1 * TILE_SZ) as i32,
        );
        !((self.position.0 + dims_px.0 as i32) < screen_pos.0
            || self.position.0 > screen_pos.0 + screen_dim.0
            || (self.position.1 + dims_px.1 as i32) < screen_pos.1
            || self.position.1 > screen_pos.1 + screen_dim.1)
    }
}
