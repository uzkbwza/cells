use crate::cell::*;
use crate::util;
use crate::map2d::*;
use crate::{WIDTH, HEIGHT, BORDERS, Error};
use rand::prelude::*;
use std::iter::Flatten;
use std::slice::Iter;

type CellMap = Map2d<Cell>;

pub struct SandApi {
    x: i32,
    y: i32,
    pub width: i32,
    pub height: i32,
    borders: bool,
    paused: bool,
    map: CellMap,
    pub highlighted: sdl2::rect::Point,
    cloned_cells: Zone<Species>,
}

pub struct Neighbor {
    pub cell: Cell,
    pub dx: i32,
    pub dy: i32,
}

impl SandApi {
    pub fn new() -> SandApi {
        let mut map = Map2d::filled_with(EMPTY, WIDTH as i32, HEIGHT as i32);
        let mut api = SandApi {
            x: 0,
            y: 0,
            width: map.width,
            height: map.height,
            paused: false,
            map,
            borders: BORDERS,
            highlighted: sdl2::rect::Point::new(0, 0),
            cloned_cells: Zone::new(),
        };
        if api.borders {
            //walls
            for y in 0..HEIGHT as i32 {
                api.map.set_point(0, y, Cell::new(Species::Border)).unwrap();
                api.map.set_point((WIDTH - 1) as i32, y, Cell::new(Species::Border)).unwrap();
            }

            // floor/ceiling
            for x in 0..WIDTH as i32 {
                api.map.set_point(x, 0, Cell::new(Species::Border)).unwrap();
                api.map.set_point(x, (HEIGHT - 1) as i32, Cell::new(Species::Border)).unwrap();
            }
        }
        api
    }
    pub fn init(&mut self) {
        // wall
        *self = Self::new();
    }

    pub fn store_cloned_cell(&mut self, species: Species) -> Option<CloneId> {
        self.cloned_cells.insert(species)
    }

    pub fn get_cloned_cell(&mut self, id: CloneId) -> Cell {
        Cell::new(self.cloned_cells.get(id).unwrap())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        if self.paused { 
            return Ok(()) 
        }

        for y in 0..HEIGHT {
            let y = HEIGHT - y;
            // bias from left to right every even row...
            for x in (0..WIDTH) {
                let mut x = x;
                if y % 2 == 0 {
                    x = WIDTH - x
                }
                self.set_cursor(x as i32, y as i32);
                self.update_heat()?;
                self.update_cell()?;
            }
        }

        // keep track of cloned cells
        let mut clone_ids = Vec::new();
        for x in 0..WIDTH{
            for y in 0..HEIGHT {
                let cell = self.get_absolute(x as i32, y as i32)?;
                if let Species::Clone(Some(id)) = cell.species {
                    clone_ids.push(id);
                }
            }
        }
        for i in 0..self.cloned_cells.len() as u16 {
            if !clone_ids.contains(&i) {
                self.cloned_cells.remove(i.into());
            }
        }

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
        if !self.borders {
            for y in 0..self.height {
                self.set(0, y, EMPTY)?;
                self.set(self.width as i32 - 1, y, EMPTY)?;
            }
            for x in 0..self.width {
                self.set(x, 0, EMPTY)?;
                self.set(x, self.height as i32 - 1, EMPTY)?;
            }
        }

        Ok(())
    }

    fn update_heat(&mut self) -> Result<(), Error> {
        use Species::*;
        let mut cell = self.get(0, 0)?;
        if matches!(cell.species, Empty | Clone(_) | Border ) {
            return Ok(())
        }
        let mut rng = thread_rng();
        if cell.is_flammable() {
            if cell.heat >= 3000 {
                cell.species = BlueFire
            } else if cell.heat >= 800 {
                cell.species = Fire
            }
        }
        let mut alone = true;
        let neighbors = self.neighbors()?;
        for n in neighbors.iter() {
            let mut neighbor = n.cell;
            if matches!(neighbor.species, Empty | Border) {
                continue
            } else {
                alone = false;
            }
            let mut changed = false;
            if cell.heat > neighbor.heat + 10 {
                cell.heat -= 10;
                neighbor.heat += 10;
                changed = true;
            } else if cell.heat == neighbor.heat {
                if rng.gen_bool(0.4) {
                    cell.heat -= 1;
                    neighbor.heat -= 1;
                    changed = true;
                }
            }
            if changed {
                self.set(n.dx,n.dy,neighbor)?;
            }
        }
        if alone && cell.heat > cell.species.starting_temp() {
            cell.heat -= 3;
        }
        if cell.heat < cell.species.starting_temp() {
            cell.heat += 1;
        }
        self.set(0,0,cell)
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

        if cell.turns_to_lava() && cell.heat > 1600 {
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
            Lava => update_lava(self, cell)?,
            Steam => update_steam(self, cell)?,
            Salt => update_salt(self, cell)?,
            SaltWater => update_salt_water(self, cell)?,
            Fire => update_fire(self, cell)?,
            BlueFire => update_fire(self, cell)?,
            Ice => update_ice(self, cell)?,
            Clone(_) => update_clone(self, cell)?,
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

    pub fn get_absolute(&self, x: i32, y: i32) -> Result<Cell, Error> {
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

struct Zone<T: PartialEq> {
    contents: Vec<Option<T>>
}

impl<T: PartialEq> Zone<T> {
    pub fn new() -> Self {
        Zone {
            contents: Vec::new(),
        }
    }
    pub fn insert(&mut self, t: T) -> Option<u16> {
        if self.contents.is_empty() {
            self.contents.push(Some(t));
            Some(0)
        } else {
            let index = self.contents.iter().enumerate().find(|(_, x)| **x == None);
            match index {
                Some((i, _)) => {
                    self.contents[i] = Some(t);
                    Some(i as u16)
                },
                None => {
                    if self.len() < i16::max_value() as usize {
                        self.contents.push(Some(t));
                        Some(self.contents.len() as u16 - 1)
                    } else {
                        None
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }

    pub fn get(&self, i: u16) -> &Option<T> {
        &self.contents[i as usize]
    }
    pub fn remove(&mut self, i: u16) -> Option<T> {
        let mut val = None;
        std::mem::swap(&mut val, &mut self.contents[i as usize]);
        val
    }
}
