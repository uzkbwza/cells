use crate::cell::*;
use crate::api::*;
use crate::Error;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use crate::{util, Controls};

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
            Color::RGB(150 - wetness * 13, 70 - wetness * 10, 33 - wetness * 5),
        Species::Acid => Color::RGB(0, 255, 100),
        Species::Wall => Color::GRAY,
        Species::Grass | Species::GrassTip => Color::GREEN,
        Species::Flower(c) => c,
        Species::Soil => Color::RGB(40, 20, 10),
        Species::Border => Color::BLACK,
        Species::Empty => Color::BLACK,
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

fn render_cell<T: RenderTarget>(cell: CellRenderInfo, canvas: &mut Canvas<T>) 
-> Result<(), Error> {
    let mut color = species_color(cell.species);
    color.r = apply_grain(color.r, cell.grain, 40);
    color.g = apply_grain(color.g, cell.grain, 40);
    color.b = apply_grain(color.b, cell.grain, 40);
    canvas.set_draw_color(color);
    canvas.draw_point(cell.point)?;
    Ok(())
}

pub fn render<T: RenderTarget>(api: &mut SandApi, canvas: &mut Canvas<T>) 
-> Result<(), Error> {
    for x in 0..api.width {
        for y in 0..api.height {
            api.set_cursor(x, y);
            let cell = api.get(0, 0).unwrap();
            if cell != EMPTY {
                let cell_render_info = CellRenderInfo::from_cell(x, y, &cell);
                render_cell(cell_render_info, canvas)?;
                canvas.set_draw_color(Color::BLACK);
            }
        }
    } 
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
    for a in -(radius + 1)..(radius + 1) {
        for b in -(radius + 1)..(radius + 1) {
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
