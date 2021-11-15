use rand::prelude::*;
use rand::seq::SliceRandom;
use crate::api::*;
use crate::Error;
use crate::util::line;
use sdl2::pixels::Color;
use std::cmp;
use sdl2::hint::get_video_minimize_on_focus_loss;
use crate::cell::Species::Empty;
use std::hint::unreachable_unchecked;

type Wetness = u8;
type Height = u8;
pub type CloneId = u16;

#[derive(Clone, Debug, Copy, PartialEq, Hash)]
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
    Stone,
    Fire,
    BlueFire,
    Ice,
    Clone(Option<CloneId>),
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

    pub fn starting_temp(&self) -> i16 {
        use Species::*;
        match self {
            Lava => 5000,
            Fire => 800,
            BlueFire => 3000,
            Ice => -160,
            _    => 20,
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
        // println!("{}", std::mem::size_of::<Cell>());
        let mut rng = thread_rng();
        let heat = species.starting_temp();

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
        matches!(self.species, Empty)
    }
    
    pub fn is_solid(&self) -> bool {
        use Species::*;
        matches!(self.species, Wall | Border | Stone | Ice)
    }

    pub fn is_gas(&self) -> bool {
        use Species::*;
        matches!(self.species, Steam)
    }

    pub fn is_liquid(&self) -> bool {
        use Species::*;
        matches!(self.species, Water | Acid | Lava | SaltWater)
    }

    pub fn is_corrodable(&self) -> bool {
        use Species::*;
        !self.is_gas() && !matches!(self.species, Empty | Wall | Border | Acid | Clone(_) )
    }

    pub fn liquid_destroyable(&self) -> bool {
        use Species::*;
        matches!(self.species, Grass | GrassTip | Flower(_))
    }

    pub fn turns_to_lava(&self) -> bool {
        use Species::*;
        matches!(self.species, Sand | Mud(_) | Salt | Soil | Stone)
    }

    pub fn is_flammable(&self) -> bool {
        use Species::*;
        matches!(self.species, Grass | GrassTip | Flower(_))
    }

    pub fn douses_fire(&self) -> bool {
        use Species::*;
        !self.is_solid() && !self.is_flammable() && !matches!(self.species, Empty | Fire | BlueFire )
    }

    pub fn is_cold(&self) -> bool {
        use Species::*;
        matches!(self.species, Ice)
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
    pub fn clone() -> Cell { Cell::new (Species::Clone(None))}
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
    let path = line(0, 0, x, y);
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
    if rng.gen_bool(0.9) {
        if go_toward(api, 0, 1, cell)? {
            return Ok(())
        }
    } else {
        if go_toward(api, dx, 1, cell)? {
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
        if below.is_fluid() && below.species != Species::Lava {
            api.swap(0, 1, cell)?;
            return Ok(())
        }
    } 

    if let Ok(alternate) = api.get(dx, 1) {
        if alternate.is_fluid() && alternate.species != Species::Lava {
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
            if absorb_probability < 10 && neighbor.dy < 0 {
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
    let mut rng = thread_rng();
    if cell.heat >= 100 {
        cell.species = Species::Steam;
        return api.set(0, 0, cell)
    } else if cell.heat <= 0 {
        cell.species = Species::Ice;
        return api.set(0, 0, cell)
    }
    // ride underneath surfaces
    let mut dirs = [1, -1];
    dirs.shuffle(&mut rng);
    for dx in dirs.iter() {
        let dx = *dx;
        if api.get(dx, 0)?.is_solid() && !api.get(0, -1)?.is_solid() && api.is_empty(dx, 1) {
            return api.swap(dx, 1, cell);
        } else if api.get(dx * 2, 0)?.is_solid() && api.get(dx * 2, -1)?.is_solid() && api.is_empty(dx, 0) && api.get(-1, 0)?.is_solid() {
            return api.swap(dx, 0, cell);
        } else if api.get(0, -1)?.is_solid() && api.is_empty(dx, 0) && rng.gen_bool(0.4) {
            return Ok(())
        }
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
        if cell.heat > 100 && rng.gen_bool(0.2) {
            if wetness > 1 {
                cell.species = Mud(wetness - 1);
            } else {
                cell.species = Sand;
                api.set(0, 0, cell)?;
                return Ok(())
            }
        }
        if (wetness >= 1
        && wetness < max_wetness
        && rng.gen::<u32>() % 100 < 9)
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
                    && rng.gen::<u32>() % 100 < 20 {
                        neighbor.cell.species = Mud (neighbor_wetness + 1);
                        api.set(neighbor.dx, neighbor.dy, neighbor.cell)?;
                        cell.species = Mud (own_wetness - 1 );
                        api.set(0, 0, cell)?;
                    }
                }

                Water => {
                    if own_wetness < max_wetness && rng.gen_bool(0.2) {
                        if rng.gen_bool(0.02) {
                            api.set(neighbor.dx, neighbor.dy, EMPTY)?;
                        }
                        cell.species = Mud (own_wetness + 1 );
                        api.set(0, 0, cell)?;
                    } else if neighbor.dy == -1 
                    && rng.gen::<u32>() % 1000 < 30 
                    && api.get(-1, -1)?.species != Species::WaterGrass(0)
                    && api.get( 1, -1)?.species != Species::WaterGrass(0) {
                        api.set(0, -1, Cell::water_grass())?;
                    }
                }

                Empty => {
                    if own_wetness >= 1 
                    && neighbor.dy >= 0 && rng.gen::<u32>() % 100 < 10 {
                        cell.species = Mud (own_wetness - 1 );
                        api.set(neighbor.dx, neighbor.dy, Cell::new(Species::Water))?;
                        api.set(0, 0, cell)?;
                    }
                    if own_wetness == max_wetness {
                        cell.species = Soil;
                        api.set(0, 0, cell)?;
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

    let root = api.get(0, 1)?.species;
    if root != Species::Soil && root != Species::Grass && rng.gen_bool(0.02) {
        api.set(0,0,EMPTY)?;

    } else {
        if api.is_empty(0, -1) {
            if rng.gen::<u32>() % 100 < 75 {
                api.set(0, -1, Cell::new(Species::Grass))?;
            } else {
                api.set(0, -1, Cell::new(Species::GrassTip))?;
            }
        }
    }

    update_coarse(api, cell)?;
    Ok(())
}

pub fn update_grass_tip(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    update_coarse(api, cell)?;
    let bloom_probability = rng.gen::<u32>() % 1000;
    if bloom_probability < 1 {
        api.set(0, 0, Cell::flower())?;
    }
    
    let root = api.get(0, 1)?.species;
    if root != Species::Soil && root != Species::Grass && rng.gen_bool(0.5) {
        if root == Species::GrassTip {
            api.set(0,0,EMPTY)?;
        } else {
            cell.species = Species::Grass;
            api.set(0,0,cell)?;
        }
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
    let max_height = 30;
    let mut rng = thread_rng();
    let dx = *[1, 0, -1].choose(&mut rng).unwrap();


    if api.neighbors()?.iter().all(|n| n.cell.species != Species::Water) && rng.gen::<u32>() % 100 < 1 {
        api.set(0, 0, Cell::mud())?;
        return Ok(())
    }

    if api.neighbors()?.iter().any(|n| n.cell == EMPTY ) && rng.gen::<u32>() % 100 < 1 {
        api.set(0, 0, Cell::new(Species::Water))?;
        return Ok(())
    }

    if api.neighbors()?.iter().filter(|n| n.dy > 0).all(|n|  
        {
            !matches!(n.cell.species, Species::WaterGrass(_) | Species::Mud(_))
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
                if rng.gen_bool(0.5) {
                    api.set(dx, -1, cell)?;
                }
                if rng.gen_bool(0.2) {
                    api.set(-dx, -1, cell)?;
                }
            }
        }
    }

    Ok(())
}

pub fn update_lava(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    if cell.heat < 1600 && rng.gen::<u32>() % 100 < 1 {
        cell.species = Species::Stone;
        return api.set(0, 0, cell);
    }
    for n in api.neighbors()?.iter_mut() {
        if n.cell.heat < cell.heat && n.cell != EMPTY {
            n.cell.heat = cmp::min(n.cell.heat + 30, cell.heat);
            cell.heat -= 5;
            api.set(n.dx, n.dy, n.cell)?;
            api.set(0, 0, cell)?;
        }
    }
    if rng.gen::<u32>() % 100 < 40 {
        update_liquid(api, cell)?;
    } else {
        update_powder(api, cell)?;
    }
    Ok(())
}


pub fn update_steam(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    let mut rng = thread_rng();
    let dx = *[1, 0, 0, -1].choose(&mut rng).unwrap();
    
    if rng.gen::<u32>() % 1000 < 5 {
        return Ok(())
    }

    if cell.heat < 100 && rng.gen_bool(0.1) {
        if rng.gen_bool(0.6) {
            cell.species = Species::Water;
            api.set(0, 0, cell)?; 
            return Ok(())
        } else {
            api.set(0, 0, EMPTY)?;
            return Ok(())
        } 
    } 

    if rng.gen::<i16>() % 100 < (70 - (cell.heat - 100))  {
        let dy = *[1, 0, 0, 0, 0, 0, -1, -1].choose(&mut rng).unwrap();
        if api.is_empty(dx, dy) {
            api.swap(dx, dy, cell)?;
        }
        return Ok(())
    }

    if api.is_empty(dx, -1) || api.get(dx, -1)?.is_gas() {
        api.swap(dx, -1, cell)?;
        return Ok(());
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

pub fn update_fire(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    use Species::*;
    let mut rng = thread_rng();
    let mut dx = *[1, 0, 0, -1].choose(&mut rng).unwrap();
    if cell.species == BlueFire && cell.heat < 2600 && rng.gen_bool(0.02) {
        cell.species = Fire;
    }
    if cell.heat < 600 {
        api.set(0, 0, EMPTY)?;
        return Ok(());
    }

    let mut moved = false;

    for dx in -1..=1 {
        if !api.is_empty(dx, -1) {
            let mut neighbor = api.get(dx, -1)?;
            if neighbor.heat < cell.heat {
                neighbor.heat += match cell.species {
                    Fire => 20,
                    BlueFire => 22,
                    _ => unreachable!()
               };
            }
            api.set(dx, -1, neighbor)?;
        }
    }

    for mut n in api.neighbors()?.iter_mut().filter(|n| n.cell.is_flammable()) {
        if matches!(n.cell.species, Fire | BlueFire | Lava) && rng.gen_bool(0.2) {
            n.cell.species = Fire;
            n.cell.heat = 800;
            api.set(n.dx, n.dy, n.cell)?;
        }
    }

    if (api.get(0, -1)?.douses_fire() && rng.gen_bool(0.4)) || (api.get(0, -1)?.is_solid() && rng.gen_bool(0.2)){
        return api.set(0, 0, EMPTY)
    }

    cell.grain = cell.grain.overflowing_add(rng.gen_range(1..20)).0;
    let mut dy= 0;
    if api.is_empty(dx, -1) {
        dy = -1;
        moved = true;
    } else if api.is_empty(dx, 0) {
            moved = true;
    } else if rng.gen_bool(0.01) {
        api.set(0, 0, EMPTY)?;
        return Ok(())
    } else {
        dx = 0
    }

    if rng.gen_bool(0.5) && moved {
        cell.heat -= match cell.heat {
            x if x <= 800 => 16,
            x if x > 800 => 120,
            x if x > 2000 => 180,
            _ => unreachable!(),
        };
    }
    api.swap(dx, dy, cell)
}

pub fn update_clone(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    use Species::*;
    let mut rng = thread_rng();
    let mut neighbors = api.neighbors()?;
    if let Clone(contents) = cell.species {
        if contents == None {
            for n in neighbors.iter() {
                if !matches!(n.cell.species, Clone(_) | Empty | Border) {
                    cell.species = Clone(api.store_cloned_cell(n.cell.species));
                    api.set(0, 0, cell)?;
                    break;
                }
            }
        }
        if let Some(id) = contents {
            let cloned = api.get_cloned_cell(id);
            cell.heat = cloned.species.starting_temp();
            for n in api.neighbors()?.iter() {
                let mut neighbor = n.cell;
                if n.cell == EMPTY && rng.gen_bool(0.05) {
                    api.set(n.dx, n.dy, cloned)?;
                } else if neighbor.species == Clone(None) && rng.gen_bool(0.1) {
                    neighbor.species = cell.species;
                    api.set(n.dx, n.dy, neighbor)?;
                }
            }
        }
    }
    api.set(0, 0, cell)?;
    Ok(())
}

pub fn update_ice(api: &mut SandApi, mut cell: Cell) -> Result<(), Error> {
    if cell.heat > 0 {
        cell.species = Species::Water;
        api.set(0, 0, cell)?;
    }
    Ok(())
}
