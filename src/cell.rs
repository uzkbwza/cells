use rand::prelude::*;
use rand::seq::SliceRandom;
use crate::api::*;
use crate::Error;
use crate::util::line;
use sdl2::pixels::Color;
use std::cmp;

type Wetness = u8;
type Height = u8;

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
    WaterGrass(Height),
    Flower(Color),
    Salt,
    SaltWater,
    Steam,
    Lava,
}

impl Species {
    pub fn decr(&mut self) {
        match self {
            Self::Mud(wetness) => {
                if *wetness > 0 {
                    *wetness -= 1;
                }
            }
            Self::WaterGrass(height) => {
                if *height > 0 {
                    *height -= 1;
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
            Self::WaterGrass(height) => {
                if *height < 255 {
                    *height += 1;
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
    pub heat: i16,
    pub grain: u8,
}

impl Cell {
    const MAX_WETNESS: u8 = 2;

    pub fn new(species: Species) -> Self {
        use Species::*;
        let mut rng = thread_rng();

        let heat = match species {
            Lava => 2000,
            _    => 20,
        };

        Cell {
            species,
            clock: false,
            heat, 
            grain: rng.gen(),
        }
    }

    pub fn is_fluid(&self) -> bool {
        use Species::*;
        if self.is_liquid() || self.is_gas() {
            return true;
        }
        match self.species {
           Empty => true, 
           _     => false
        }
    }

    pub fn is_gas(&self) -> bool {
        use Species::*;
        match self.species{
            Steam => true,
            _     => false,
        }
    }

    pub fn is_liquid(&self) -> bool {
        use Species::*;
        match self.species {
           Water => true, 
           Acid => true,
           Lava => true,
           SaltWater => true,
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

    pub fn water_grass() -> Cell {
        Cell::new(Species::WaterGrass(0))
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
    heat: 0,
    grain: 0,
};

fn go_toward(api: &mut SandApi, x: i32, y: i32, cell: Cell) -> Result<bool, Error> {
    // tries to go as far as possible towards the next point
    let mut path = line(0, 0, x, y);
    let mut moved = false;
    let mut swap_point = sdl2::rect::Point::new(0, 0);
    for i in 1..path.len() {
        let dx = path[i].x;
        let dy = path[i].y;
        if api.is_empty(dx, dy) {
            moved = true;
            swap_point = path[i];
        } else {
            break;
        }
    }

    if moved {
        api.swap(swap_point.x, swap_point.y, cell)?;
    }
    //api.set_cursor(start_x, start_y);
    Ok(moved)
}

pub fn update_liquid(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let dirs = [1, 0, -1];
    let dx = *dirs.choose(&mut rng).unwrap();

    let neighbors = api.neighbors()?;
    let neighbor = neighbors.choose(&mut rng).unwrap();
    if neighbor.cell.liquid_destroyable() && neighbor.dy >= 0 {
        api.set(neighbor.dx, neighbor.dy, EMPTY)?;
    }
    fn can_swap(api: &mut SandApi, x: i32, y: i32) -> bool {
        api.is_empty(x, y) || api.get(x, y).unwrap().is_gas()
    }
    
    // fall down
    if go_toward(api, dx, 2, cell)? {
        return Ok(())
    } else {
        if go_toward(api, dx*2, 0, cell)? {
            return Ok(())
        }
    }
    if can_swap(api, 0, 1) {
        api.swap(0, 1, cell)?;
        return Ok(())
    } else if can_swap(api, dx, 1) {
        api.swap(dx, 1, cell)?;
        return Ok(())
    } else if can_swap(api, dx, 0) {
        api.swap(dx, 0, cell)?;
        return Ok(())
    } else if can_swap(api, -dx, 1) {
        api.swap(-dx, 1, cell)?;
        return Ok(())
    } else if can_swap(api, -dx, 0) {
        api.swap(-dx, 0, cell)?;
        return Ok(())
    }
    
    let swap_probability = rng.gen::<u32>() % 100;
    if swap_probability < 10 {
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

pub fn update_gas(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let dirs = [-1, 0, 1];
    let dx = *dirs.choose(&mut rng).unwrap();
    let dy = *dirs.choose(&mut rng).unwrap();
    if api.get(dx, dy)?.is_gas() || api.is_empty(dx, dy) {
        api.swap(dx, dy, cell)?;
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
    let mut neighbors = api.neighbors()?;
    for neighbor in neighbors.iter_mut() {
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

pub fn update_water(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    if cell.heat >= 100 {
        cell.species = Species::Steam;
        api.set(0, 0, cell)?;
    }
    update_liquid(api, cell)
}

pub fn update_mud(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    use Species::*;
    let max_wetness = Cell::MAX_WETNESS;
    let mut rng = thread_rng();

    // absorb water, and then overflow into other sand blocks if full
    let mut neighbors = api.neighbors()?;
    neighbors.shuffle(&mut rng);

    if let Mud(wetness)  = cell.species {
        if (wetness >= 1
        && wetness < max_wetness
        && rng.gen::<u32>() % 100 < 2)
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

                Sand => {
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

                Water => {
                    if own_wetness < max_wetness {  
                        api.set(neighbor.dx, neighbor.dy, EMPTY)?;
                        cell.species = Mud (own_wetness + 1 );
                        api.set(0, 0, cell)?;
                    } else {
                        if neighbor.dy == -1 
                        && rng.gen::<u32>() % 1000 < 30 
                        && api.get(-1, -1)?.species != Species::WaterGrass(0)
                        && api.get( 1, -1)?.species != Species::WaterGrass(0) {
                            api.set(0, -1, Cell::water_grass())?;
                        }
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
    && rng.gen::<u32>() % 100 < 1 {
        api.set(0, -1, Cell::new(Species::Grass))?; 
    }
    // occasionally absorb water, become mud
    use Species::*;
    let mut neighbors = api.neighbors()?;
    let absorb_probability = rng.gen::<u32>() % 1000;
    for neighbor in neighbors.iter_mut() {
        match neighbor.cell.species {
            Water => { 
                if absorb_probability < 55 && neighbor.dy < 0 {
                    api.set(neighbor.dx, neighbor.dy, EMPTY)?;
                    api.set(0, 0, Cell::mud())?; 
                }
            }
            Mud ( wetness ) => {
                if wetness >= Cell::MAX_WETNESS && absorb_probability < 10 {
                    api.set(neighbor.dx, neighbor.dy, Cell::new(Mud(wetness/2)))?; 
                    api.set(0, 0, Cell::new(Mud(wetness/2)))?; 
                }
            }
            _ => {}
        }
    }
    Ok(())
}



pub fn update_grass(api: &mut SandApi, cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    if api.is_empty(0, -1) {
        if rng.gen::<u32>() % 100 < 75 {
            api.set(0, -1, Cell::new(Species::Grass))?;
        } else {
            api.set(0, -1, Cell::new(Species::GrassTip))?;
        }
    }

    let root = api.get(0, 1)?.species;
    if root != Species::Soil && root != Species::Grass {
        api.set(0,0,EMPTY)?;
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
    
    let root = api.get(0, 1)?.species;
    if root != Species::Soil && root != Species::Grass {
        api.set(0,0,EMPTY)?;
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

pub fn update_water_grass(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let max_height = 5;
    let mut rng = thread_rng();
    let dx = *[1, 0, -1].choose(&mut rng).unwrap();


    if api.neighbors()?.iter().all(|n| n.cell.species != Species::Water) {
        if rng.gen::<u32>() % 100 < 1 {
            api.set(0, 0, Cell::mud())?;
            return Ok(())
        }
    }

    if api.neighbors()?.iter().any(|n| n.cell == EMPTY ) {
        if rng.gen::<u32>() % 100 < 1 {
            api.set(0, 0, Cell::new(Species::Water))?;
            return Ok(())
        }
    }

    if api.neighbors()?.iter().filter(|n| n.dy > 0).all(|n|  
        {
            match n.cell.species {
                Species::WaterGrass(_) | Species::Mud(_) => false,
                _ => true,
            }
    }) {
        update_coarse(api, cell)?;
    }

    if api.get(1, -1)?.species == Species::Water
    && api.get(-1, -1)?.species == Species::Water 
    && api.get(0, -1)?.species == Species::Water 
    && api.get(1, 0)?.species == Species::Water  
    && api.get(-1, 0)?.species == Species::Water {
        if let Species::WaterGrass(height) = cell.species {
            if height < max_height {
                cell.species.incr();
                api.set(dx, -1, cell)?;
            }
        }
    }

    Ok(())
}

pub fn update_lava(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    if cell.heat < 150 && rng.gen::<u32>() % 100 < 1 {
        cell.species = Species::Sand;
        return api.set(0, 0, cell);
    }
    for n in api.neighbors()?.iter_mut() {
        if n.cell.heat < cell.heat && n.cell != EMPTY {
            n.cell.heat = cmp::min(n.cell.heat + 5, cell.heat);
            cell.heat -= 5;
            api.set(n.dx, n.dy, n.cell)?;
            api.set(0, 0, cell)?;
        }
    }
    if rng.gen::<u32>() % 100 < 90 {
        update_liquid(api, cell)?;
    } else {
        update_powder(api, cell)?;
    }
    Ok(())
}


pub fn update_steam(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let dx = *[1, 0, -1].choose(&mut rng).unwrap();
 
    if rng.gen::<u32>() % 1000 < 5 {
        api.set(0, 0, EMPTY)?;
        return Ok(())
    }

    if api.is_empty(dx, -1) || api.get(dx, -1)?.is_gas() {
        api.swap(dx, -1, cell)?;
        return Ok(());
    } 

    if cell.heat < 100 {
        cell.species = Species::Water;
        api.set(0, 0, cell)?; 
        return Ok(())
    }


    if rng.gen::<u32>() % 100 < 50 {
        cell.heat -= 10;
    }
    update_gas(api, cell)
}

pub fn update_salt(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let mut neighbors = api.neighbors()?;
    neighbors.shuffle(&mut rng);
    for n in neighbors.iter_mut() {
        if n.cell.species == Species::Water && rng.gen::<u32>() % 100 < 5 {
            cell.species = Species::SaltWater;
            api.set(n.dx, n.dy, EMPTY)?;
            api.set(0, 0, cell)?;
            return Ok(())
        }
    }
    update_powder(api, cell)
}

pub fn update_salt_water(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    if cell.heat >= 101 {
        cell.species = Species::Steam;
        api.set(0, -1, cell)?;
        cell.species = Species::Salt;
        api.set(0, 0, cell)?; 
        return Ok(())
    }
    update_liquid(api, cell)
}
