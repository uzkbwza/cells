use rand::prelude::*;
use rand::seq::SliceRandom;
use crate::api::*;
use crate::Error;
use sdl2::pixels::Color;

type Wetness = u8;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Species {
    Empty,
    Border,
    Wall,
    Sand, 
    Water,
    Mud (Wetness),
    Acid,
    Soil,
    Grass,
    GrassTip,
    Flower(Color),
}

impl Species {
    pub fn decr(&mut self) {
        match self {
            Self::Mud(wetness) => {
                if *wetness > 0 {
                    *wetness -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn incr(&mut self) {
        match self {
            Self::Mud(wetness) => {
                if *wetness < 255 {
                    *wetness += 1;
                }
            }
            _ => {}
        }
    }
}



#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Cell {
    pub species: Species,
    pub clock: bool,
    pub grain: u8,
}

impl Cell {
    pub fn new(species: Species) -> Self {
        let mut rng = thread_rng();
        Cell {
            species,
            clock: false,
            grain: rng.gen(),
        }
    }

    pub fn is_fluid(&self) -> bool {
        use Species::*;
        match self.species {
           Water | Empty => true, 
           _     => false
        }
    }

    pub fn is_liquid(&self) -> bool {
        use Species::*;
        match self.species {
           Water => true, 
           _     => false
        }
    }

    pub fn is_corrodable(&self) -> bool {
        use Species::*;
        match self.species { 
            Empty | Wall | Border | Acid => false,
            _ => true
        }
    }

    pub fn liquid_destroyable(&self) -> bool {
        use Species::*;
        match self.species { 
            Grass | GrassTip | Flower(_) => true,
            _ => false, 
        }
    }

    // resets grain on cell. this is basically just for using the brush, 
    // so it doesn't look weird.
    pub fn regrain(&mut self) {
        let mut rng = thread_rng();
        self.grain = rng.gen();
    }

    pub fn mud() -> Cell {
        Cell::new(Species::Mud(0))
    }

    pub fn flower() -> Cell {
        let mut rng = thread_rng();
        let colors = [
            Color::RED,
            Color::BLUE,
            Color::YELLOW,
            Color::MAGENTA,
            Color::WHITE,
        ];
        let color = *colors.choose(&mut rng).unwrap();
        Cell::new(Species::Flower(color))
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new(Species::Empty)
    }
}
// species constants
pub const EMPTY: Cell = Cell {
    species: Species::Empty,
    clock: false,
    grain: 0,
};

pub fn update_liquid(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let dirs = [1, 0, -1];
    let dx = *dirs.choose(&mut rng).unwrap();

    let neighbors = api.neighbors()?;
    let neighbor = neighbors.choose(&mut rng).unwrap();
    if neighbor.cell.liquid_destroyable() && neighbor.dy >= 0 {
        api.set(neighbor.dx, neighbor.dy, EMPTY)?;
    }
    // fall down
    if api.is_empty(0, 1) {
        api.swap(0, 1, cell)?;
        return Ok(())
    } else if api.is_empty(dx, 1) {
        api.swap(dx, 1, cell)?;
        return Ok(())
    
    } else if api.is_empty(dx, 0) {
        api.swap(dx, 0, cell)?;
        return Ok(())
    } else if api.is_empty(-dx, 1) {
        api.swap(-dx, 1, cell)?;
        return Ok(())
    } else if api.is_empty(-dx, 0) {
        api.swap(-dx, 0, cell)?;
        return Ok(())
    }
    
    let swap_probability = rng.gen::<u32>() % 100;
    if swap_probability < 1 {
        let mut neighbors = api.neighbors()?;
        neighbors.shuffle(&mut rng);
        for neighbor in neighbors.iter() {
            if neighbor.cell.is_liquid() {
                api.swap(neighbor.dx, neighbor.dy, cell)?;
                break;
            }
        }
    }

    Ok(())
}

pub fn update_coarse(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    if let Ok(below) = api.get(0,1) {
        if below.is_fluid() {
            api.swap(0, 1, cell)?;
            return Ok(())
        }
    }
    Ok(())
}

pub fn update_powder(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let fall_probability = rng.gen::<u32>() % 100;
    if fall_probability < 10 {
        return Ok(()) // stay in place this frame
    }

    let dirs = [1, 0, -1];
    let dx = *dirs.choose(&mut rng).unwrap();

    // try to fall below, or otherwise to the side
    if api.is_empty(0, 1) && api.is_empty(0, 2) {
        api.swap(0, 2, cell)?;
        return Ok(())
    }

    if let Ok(below) = api.get(0,1) {
        if below.is_fluid() {
            api.swap(0, 1, cell)?;
            return Ok(())
        }
    } 

    if let Ok(alternate) = api.get(dx, 1) {
        if alternate.is_fluid() {
            api.swap(dx, 1, cell)?;
            return Ok(())
        }
    } 
    Ok(())
}

pub fn update_sand(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let fall_probability = rng.gen::<u32>() % 100;
    if fall_probability < 10 {
        return Ok(()) // stay in place this frame
    }
    // occasionally absorb water, become mud
    use Species::*;
    for neighbor in api.neighbors()?.iter_mut() {
        if let Water{ .. } = neighbor.cell.species { 
            let absorb_probability = rng.gen::<u32>() % 100;
            if absorb_probability < 1 && neighbor.dy < 0 {
                api.set(neighbor.dx, neighbor.dy, EMPTY)?;
                api.set(0, 0, Cell::mud())?; 
            }
        }
    }
    update_powder(api, cell)?;
    Ok(())
}


pub fn update_acid(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    update_liquid(api, cell)?;
    let mut rng = thread_rng();
    for neighbor in api.neighbors()?.iter() {
        if rng.gen::<u32>() % 100 < 3 && neighbor.cell.is_corrodable() {
            api.set(neighbor.dx, neighbor.dy, EMPTY)?;
            if rng.gen::<u32>() % 100 < 30 {
                api.set(0, 0, EMPTY)?;
            }
        }
    }
    Ok(())
}

pub fn update_water(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    update_liquid(api, cell)
}

pub fn update_mud(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    use Species::*;
    let max_wetness = 5;
    let mut rng = thread_rng();

    // absorb water, and then overflow into other sand blocks if full
    let mut neighbors = api.neighbors()?;
    neighbors.shuffle(&mut rng);

    if let Mud(wetness)  = cell.species {
        if (wetness >= 1
        && wetness < max_wetness
        && rng.gen::<u32>() % 1000 < 5)
        && (neighbors.iter().any(|n| n.cell.species == Empty || n.cell.species == Soil ))
        && (neighbors.iter().all(|n| n.cell.species != Water))
        {
            api.set(0, 0, Cell::new(Species::Soil))?;
            return Ok(());
        }
    }

    for neighbor in neighbors.iter_mut() {
        let absorb_probability = rng.gen::<u32>() % 100;

        if absorb_probability > 15 { 
            continue 
        }

        if let Mud(wetness)  = cell.species {
            let own_wetness = wetness;
            match neighbor.cell.species { 

                Sand {..} => {
                    if own_wetness >= 1 && neighbor.dy >= 0 {
                        api.set(neighbor.dx, neighbor.dy, Cell::mud())?;
                        cell.species = Mud(own_wetness - 1);
                        api.set(0, 0, cell)?;
                    }
                }

                Mud (wetness) => {
                    let neighbor_wetness = wetness;
                    if neighbor_wetness < max_wetness 
                    && own_wetness >= 1 
                    && neighbor.dy >= 0
                    && neighbor.dy.abs() != neighbor.dx.abs() 
                    && rng.gen::<u32>() % 100 < 30 {
                        neighbor.cell.species = Mud (neighbor_wetness + 1);
                        api.set(neighbor.dx, neighbor.dy, neighbor.cell)?;
                        cell.species = Mud (own_wetness - 1 );
                        api.set(0, 0, cell)?;
                    }
                }

                Water { .. } => {
                    if own_wetness < max_wetness {  
                        api.set(neighbor.dx, neighbor.dy, EMPTY)?;
                        cell.species = Mud (own_wetness + 1 );
                        api.set(0, 0, cell)?;
                    }
                }

                Empty => {
                    if own_wetness >= 1 
                    && neighbor.dy >= 0 {
                        if rng.gen::<u32>() % 100 < 10 {
                            cell.species = Mud (own_wetness - 1 );
                            api.set(neighbor.dx, neighbor.dy, Cell::new(Species::Water))?;
                            api.set(0, 0, cell)?;
                        }
                    }
                }
                
                _ => {}
            }
        }
    }     
    update_coarse(api, cell)?;
 
   Ok(())
}

pub fn update_soil(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    update_coarse(api, cell)?;
    let mut rng = thread_rng();
    if api.is_empty(0, -1) 
    && api.is_empty(1, -2) 
    && api.is_empty(-1, -2) 
    && rng.gen::<u32>() % 100 < 1 {
        api.set(0, -1, Cell::new(Species::Grass))?; 
    }
    Ok(())
}



pub fn update_grass(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    if api.is_empty(0, -1) {
        if rng.gen::<u32>() % 100 < 65 {
            api.set(0, -1, Cell::new(Species::Grass))?;
        } else {
            api.set(0, -1, Cell::new(Species::GrassTip))?;
        }
    }
    update_coarse(api, cell)?;
    Ok(())
}

pub fn update_grass_tip(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    update_coarse(api, cell)?;
    let bloom_probability = rng.gen::<u32>() % 1000;
    if bloom_probability < 1 {
        api.set(0, 0, Cell::flower())?;
    }
    Ok(())
}

pub fn update_flower(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    // grow from "grass"
    if api.get(0, 1)?.species == Species::Grass
    && [
        api.get(1, -1)?, 
        api.get(0, -1)?, 
        api.get(-1, -1)?, 
        api.get(0, -2)?
    ].iter().all(|n| *n == EMPTY) {
        api.set(1, -1, cell)?;
        api.set(-1, -1, cell)?;
        api.set(0, -2, cell)?;
    }

    // dont fall if supported diagonally
    if let Species::Flower(c) = api.get(0,0)?.species {
        if [api.get(-1, 1)?, api.get(1, 1)?].iter().any(|n| n.species == Species::Flower(c)) {
            return Ok(())
        }
    }
    update_coarse(api, cell)?;
    Ok(())
}
