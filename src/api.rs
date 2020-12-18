use crate::cell::*;
use crate::util;
use crate::map2d::*;
use crate::{WIDTH, HEIGHT, Error};
use rand::prelude::*;

type CellMap = Map2d<Cell>;

pub struct SandApi {
    x: i32,
    y: i32,
    pub width: i32,
    pub height: i32,
    paused: bool,
    map: CellMap,
    pub highlighted: sdl2::rect::Point,
}

pub struct Neighbor {
    pub cell: Cell,
    pub dx: i32,
    pub dy: i32,
}

impl SandApi {
    pub fn new() -> SandApi {
        let mut map = Map2d::filled_with(EMPTY, WIDTH as i32, HEIGHT as i32);



        // walls
        for y in 0..HEIGHT as i32 {
            map.set_point(0,            y, Cell::new(Species::Border)).unwrap();
            map.set_point((WIDTH-1) as i32, y, Cell::new(Species::Border)).unwrap();
        }

        // floor/ceiling 
        for x in 0..WIDTH as i32 {
            map.set_point(x,             0, Cell::new(Species::Border)).unwrap();
            map.set_point(x, (HEIGHT-1) as i32, Cell::new(Species::Border)).unwrap();
        }

        SandApi {
            x: 0,
            y: 0,
            width: map.width,
            height: map.height,
            paused: false,
            map, 
            highlighted: sdl2::rect::Point::new(0, 0),
        }
    }

    pub fn update(&mut self) -> Result<(), Error> {
        if self.paused { 
            return Ok(()) 
        }
        for y in 0..HEIGHT {
            // go from left to right every even row...
            if y % 2 == 0 {
                for x in (0..WIDTH) {
                    self.set_cursor(x as i32, y as i32);
                    self.update_cell()?;
                }
            } else {
                // and right to left every odd.
                for i in (0..WIDTH) {
                    let x = WIDTH - i;
                    self.set_cursor(x as i32, y as i32);
                    self.update_cell()?;
                }
            }
        }

        self.update_heat()?; 
        for x in 0..WIDTH{
            for y in 0..HEIGHT{
                self.set_cursor(x as i32, y as i32);
                let mut cell = self.get(0, 0)?;
                cell.clock = false; 
                self.set(0, 0, cell)?;
            }
        }
        self.x = 0;
        self.y = 0;
        Ok(())
    }

    fn update_heat(&mut self) -> Result<(), Error> {
        let mut rng = thread_rng();
        for x in 1..WIDTH - 1{
            for y in 1..HEIGHT - 1{
                self.x = x as i32;
                self.y = y as i32;
                let mut cell = self.get(0, 0)?;
                if cell != EMPTY {
                    for n in self.neighbors()?.iter() {
                        let mut neighbor = n.cell;
                        if neighbor == EMPTY {
                            continue
                        }
                        if cell.heat > neighbor.heat + 5 && cell.heat > 20 {
                            cell.heat -= 5;
                            neighbor.heat += 5; 
                        } else if cell.heat == neighbor.heat {
                            if rng.gen_bool(0.4) {
                                cell.heat -= 1;
                                neighbor.heat -= 1;
                            }
                        }
                        self.set(n.dx,n.dy,neighbor)?;
                    }
                    if self.neighbors()?.iter().filter(|n| n.cell != EMPTY).collect::<Vec<&Neighbor>>().len() == 0 && cell.heat > 20 {
                        cell.heat -= 3;
                    }
                    if cell.heat < 20 {
                        cell.heat += 1;
                    }
                }
                self.set(0,0,cell)?;
            }
        }
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn update_cell(&mut self) -> Result<(), Error> {
        use Species::*;
        let mut cell = self.get(0, 0)?;

        if cell.clock || cell == EMPTY || cell.species == Border {
            return Ok(())
        } else { 
            cell.clock = true; 
        }

        if cell.turns_to_lava() && cell.heat > 1000 {
            let mut rng = thread_rng();
            if rng.gen::<usize>() % 100 < 5 {
                cell.species = Lava;
                self.set(0, 0, cell)?;
            }
        } 

        match cell.species {
            Sand => update_sand(self, cell)?,
            Water => update_water(self, cell)?, 
            Mud(_) => update_mud(self, cell)?,
            Acid => update_acid(self, cell)?,
            Grass => update_grass(self, cell)?,
            Soil => update_soil(self, cell)?,
            GrassTip => update_grass_tip(self, cell)?,
            Flower(_) => update_flower(self, cell)?,
            WaterGrass(_) => update_water_grass(self, cell)?,
            Lava          => update_lava(self, cell)?,
            Steam         => update_steam(self, cell)?,
            Salt          => update_salt(self, cell)?,
            SaltWater     => update_salt_water(self, cell)?,
            _             => {}
        };


        Ok(())
    }

    pub fn get(&self, dx: i32, dy: i32) -> Result<Cell, Error> {
        let nx = self.x + dx;
        let ny = self.y + dy;
        let cell = self.map.retrieve(nx, ny)?;
        Ok(cell)
    }

    pub fn set(&mut self, dx: i32, dy: i32, cell: Cell) -> Result<(), Error> {
        let nx = self.x + dx;
        let ny = self.y + dy;
        self.map.set_point(nx, ny, cell)?;
        Ok(())
    }

    pub fn set_absolute(&mut self, x: i32, y: i32, cell: Cell) -> Result<(), Error> {
        self.map.set_point(x, y, cell)?;
        Ok(())
    }

    pub fn get_absolute(&mut self, x: i32, y: i32) -> Result<Cell, Error> {
        let cell = self.map.retrieve(x, y)?;
        Ok(cell)
    }

    pub fn brush(&mut self, x: i32, y: i32, radius: i32, mut cell: Cell) -> Result<(), Error> {
        if radius == 1 {
            if let Ok(c) = self.get_absolute(x, y) {
                if c == EMPTY {
                    self.set_absolute(x, y, cell)?;
                }
            }
            return Ok(());

        }
        for a in -radius..=radius {
            for b in -radius..=radius {
                let nx = a + x;
                let ny = b + y; 
                if let Ok(c) = self.get_absolute(nx, ny) {
                    if util::distance(x, y, nx, ny) <= radius as f32 && c == EMPTY {
                        cell.regrain();
                        self.set_absolute(nx, ny, cell)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn erase (&mut self, x: i32, y: i32, radius: i32) -> Result<(), Error> {
        if radius == 1 {
            if let Ok(c) = self.get_absolute(x, y) {
                if c.species != Species::Border {
                    return self.set_absolute(x, y, EMPTY)
                }
            }
        }
        for a in -radius..=radius {
            for b in -radius..=radius {
                let nx = a + x;
                let ny = b + y; 
                if let Ok(c) = self.get_absolute(nx, ny) {
                    if c.species != Species::Border && util::distance(x, y, nx, ny) <= radius as f32 {
                        self.set_absolute(nx, ny, EMPTY)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn is_empty(&self, dx: i32, dy: i32) -> bool {
        if let Ok(c) = self.get(dx, dy) {
            c.species == Species::Empty
        } else {
            false
        }
    }

    pub fn neighbors(&self) -> Result<[Neighbor; 8], Error> {
        let a = [
            Neighbor { cell: self.get(-1, -1)?, dx: -1, dy: -1 },
            Neighbor { cell: self.get(0,  -1)?, dx: 0,  dy: -1 },
            Neighbor { cell: self.get(-1,  0)?, dx: -1, dy: 0  },
            Neighbor { cell: self.get(-1,  1)?, dx: -1, dy:  1 },
            Neighbor { cell: self.get( 1, -1)?, dx:  1, dy: -1 },
            Neighbor { cell: self.get( 1,  1)?, dx:  1, dy:  1 },
            Neighbor { cell: self.get( 0,  1)?, dx:  0, dy:  1 },
            Neighbor { cell: self.get( 1,  0)?, dx:  1, dy:  0 },
        ];
        Ok(a)
    }

    pub fn swap(&mut self, dx: i32, dy: i32, cell: Cell) -> Result<(), Error> {
        let other = self.get(dx, dy)?;
        self.set(0, 0, other)?;  
        self.set(dx, dy, cell)?;
        Ok(())
    }

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn get_cursor(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Default for SandApi {
    fn default() -> Self {
        Self::new()
    }
}


