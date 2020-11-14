use crate::cell::*;
use crate::api::*;
use crate::Error;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget, Texture};
use crate::{util, Controls, WIDTH};

#[derive(Debug, Clone)]
struct CellRenderInfo {
    point: Point,
    species: Species,
    grain: u8,
}

impl CellRenderInfo {
    fn from_cell(x: i32, y: i32, cell: &Cell) -> Self {
        CellRenderInfo { 
            point: Point::new(x, y),
            species: cell.species,
            grain: cell.grain
        }
    }
}

fn species_color(species: Species) -> Color {
    match species {
        Species::Sand => Color::RGB(190, 140, 40),
        Species::Water => 
            Color::RGB(30, 100, 235),
        Species::Mud(wetness) => 
            Color::RGB(150 - wetness * 23, 70 - wetness * 20, 33 - wetness * 8),
        Species::Acid => Color::RGB(0, 255, 100),
        Species::Wall => Color::GRAY,
        Species::Grass | Species::GrassTip => Color::GREEN,
        Species::Flower(c) => c,
        Species::Soil => Color::RGB(50, 10, 10),
        Species::WaterGrass(_) => Color::RGB(10, 100, 40),
        Species::Steam => Color::RGB(90, 190, 255),
        Species::Lava => Color::RGB(255, 50, 0),
        Species::Empty => Color::RGB(0, 2, 5),
        Species::Salt => Color::RGB(254, 240, 200),
        Species::SaltWater => Color::RGB(130, 130, 220),
        Species::Border => Color::RGB(1, 1, 1),
        #[allow(unreachable_patterns)]
        _                       => Color::MAGENTA,
    }
}

fn apply_grain(mut value: u8, grain: u8, max_grain_amount: u8) -> u8 {
    if value > 255 - max_grain_amount {
        value = 255 - max_grain_amount;
    }
    value + grain % max_grain_amount
}

fn cell_color(cell: CellRenderInfo ) 
-> Color {
    let mut color = species_color(cell.species);
    if cell.species == Species::Empty {
        return color
    }
    //if color == Color::BLACK { return color }
    color.r = apply_grain(color.r, cell.grain, 40);
    color.g = apply_grain(color.g, cell.grain, 40);
    color.b = apply_grain(color.b, cell.grain, 40);
    color
}

pub fn render<T: RenderTarget>(api: &mut SandApi, canvas: &mut Canvas<T>, tex: &mut Texture) 
-> Result<(), Error> { 
    let mut v = Vec::new();
    for y in 0..api.height {
        for x in 0..api.width {
            api.set_cursor(x, y);
            let cell = api.get(0, 0).unwrap();
            let cell_render_info = CellRenderInfo::from_cell(x, y, &cell);
            let c = cell_color(cell_render_info);
            let mut unpacked = vec![c.a, c.b, c.g, c.r];
            v.append(&mut unpacked);
        }
    } 
    tex.update(None, &v, WIDTH as usize * 4).unwrap();
    canvas.copy(tex, None, None).unwrap();
    Ok(())
}

pub fn render_cursor<T: RenderTarget>(controls: &Controls, canvas: &mut Canvas<T>) -> Result<(), Error> {
    let radius = controls.radius;
    let x = controls.mouse_x;
    let y = controls.mouse_y;
    canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
    if radius == 1 {
            canvas.draw_point(Point::new(x, y))?;
            canvas.set_draw_color(Color::BLACK);
            return Ok(());
    }
    for a in -radius..=radius {
        for b in -radius..=radius {
            let nx = a + x;
            let ny = b + y; 
            if util::distance(x, y, nx, ny).round() as i32 == radius {
                canvas.draw_point(Point::new(nx, ny))?;
            }
        }
    }
    canvas.set_draw_color(Color::BLACK);
    Ok(())
}
