use sdl2::pixels::Color;
use vecmap::*;
use crate::{WIDTH, HEIGHT};
use rand::*;
use sdl2::rect::Point;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Species {
    Empty,
    Wall,
    Sand,
    Fire,
    Water,
}

#[derive(Clone, Debug)]
pub enum Error {
    Error(String)
}

type CellMap = VecMap<Cell>;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Cell {
    pub species: Species,
    pub ra: i32,
    pub rb: i32,
    pub rc: i32,
    pub clock: bool,
}

impl Cell {
    pub fn new(species: Species) -> Self {
        Cell {
            species,
            ra: 0,
            rb: 0,
            rc: 0,
            clock: false,
        }
    }
}

const EMPTY: Cell = Cell {
    species: Species::Empty,
    ra: 0,
    rb: 0,
    rc: 0,
    clock: false,
};

pub struct SandApi {
    x: i32,
    y: i32,
    map: CellMap
}

impl SandApi {
    pub fn new() -> SandApi {
        let mut map = VecMap::filled_with(EMPTY, WIDTH as i32, HEIGHT as i32);

        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let x = rng.gen::<u32>() % WIDTH;
            let y = rng.gen::<u32>() % HEIGHT;
            map.set_point(x as i32, y as i32, Cell::new(Species::Sand)).unwrap();
        }

        // walls
        for y in 0..HEIGHT as i32 {
            map.set_point(0,            y, Cell::new(Species::Wall)).unwrap();
            map.set_point((WIDTH-1) as i32, y, Cell::new(Species::Wall)).unwrap();
        }

        // floor/ceiling 
        for x in 0..WIDTH as i32 {
            map.set_point(x,             0, Cell::new(Species::Wall)).unwrap();
            map.set_point(x, (HEIGHT-1) as i32, Cell::new(Species::Wall)).unwrap();
        }

        SandApi {
            x: 0,
            y: 0,
            map, 
        }
    }

    pub fn update(&mut self) -> Result<(), Error> {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.x = x as i32;
                self.y = y as i32;
                let cell = self.get(0, 0)?;
                if cell.clock == false {
                    self.update_cell()?;
                }
            }
        }

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.x = x as i32;
                self.y = y as i32;
                let mut cell = self.get(0, 0)?;
                cell.clock = false; 
                self.set(0, 0, cell)?;
            }
        }
        Ok(())
    }

    fn update_cell(&mut self) -> Result<(), Error> {
        use Species::*;
        let cell = self.get(0, 0)?;
        if cell.clock {
            return Ok(())
        }
        match cell.species {
            Sand => update_sand(self, cell)?,
            _    => {}
        };
        Ok(())
    }

    pub fn get(&self, dx: i32, dy: i32) -> Result<Cell, Error> {
        let nx = self.x + dx;
        let ny = self.y + dy;
        match self.map.retrieve(nx, ny) {
            Ok(c) => Ok(c),
            Err(s) => Err(Error::Error(s))
        }
    }

    pub fn set(&mut self, dx: i32, dy: i32, cell: Cell) -> Result<(), Error> {
        let nx = self.x + dx;
        let ny = self.y + dy;
        match self.map.set_point(nx, ny, cell) {
            Ok(()) => Ok(()),
            Err(s) => Err(Error::Error(s))
        }
    }

    pub fn is_empty(&self, dx: i32, dy: i32) -> bool {
        match self.get(dx, dy) { 
            Ok(c) => c.species == Species::Empty,
            _     => false
        }
    }

    pub fn get_dots(&mut self) -> Vec<Point> {
        let mut v = Vec::new();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.x = x as i32;
                self.y = y as i32;
                match self.get(0, 0) {
                    Err(_) => {},
                    Ok(c) => if !c.clock {
                        match c.species {
                            Species::Empty => {},
                            _              => v.push(Point::new(x as i32, y as i32)),
                        }
                    }
                }
            }
        }
        v
    }
}

fn update_sand(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let dy = 1;
    let dx = (rng.gen::<i32>() % 2);
    if api.is_empty(dx, dy) {
        cell.clock = true;
        api.set(0, 0, EMPTY)?; 
        api.set(dx, dy, cell)?;
    }
    Ok(())
}
