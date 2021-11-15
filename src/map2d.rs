use crate::{WIDTH, HEIGHT, Error};

#[derive(Debug)]
pub struct Map2d<T> {
    pub items: Box<[T; WIDTH as usize * HEIGHT as usize]>,
    pub width: i32,
    pub height: i32,
    default: T
}

impl<T> Map2d<T> where T: Clone + Copy {
    pub fn filled_with(item: T, width: i32, height: i32) -> Self {
        let items = Box::new([item; (WIDTH * HEIGHT) as usize]);
        let default = item;
        Map2d {
            items,
            width,
            height,
            default
        }
    }

    pub fn retrieve(&self, x: i32, y: i32) -> Result<T, Error> {
        let id = self.xy_idx(x.clamp(0, -1 + WIDTH as i32), y.clamp(0, -1 + HEIGHT as i32));
        Ok(self.items[id])
    }

    pub fn set_point(&mut self, x: i32, y: i32, item: T) -> Result<(), String> {
        let id = self.xy_idx(x, y);
        if x < self.width && y < self.height && x >=0 && y >= 0 {
            self.items[id] = item;
            return Ok(());
        }
        Ok(())
    }

    pub fn reset_point(&mut self, x: i32, y: i32) -> Result<(), String> {
        self.set_point(x, y, self.default)
    }

    pub fn reset_map(&mut self) -> Result<(), String> {
        for i in 0..self.items.len() {
            let (x, y) = self.idx_xy(i);
            self.reset_point(x, y)?;
        }
        Ok(())
    }

    // stealing this from thebracket

    // We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
    // a tile. Each row is stored sequentially (so 0..80, 81..160, etc.). This takes an x/y and returns
    // the array index.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize{
        (y as usize * self.width as usize) + x as usize
    }

    // It's a great idea to have a reverse mapping for these coordinates. This is as simple as
    // index % MAP_WIDTH (mod MAP_WIDTH), and index / MAP_WIDTH
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }
}
