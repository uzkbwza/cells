extern crate sdl2; 
pub mod cell;
pub mod render;
pub mod api;
pub mod util;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
#[allow(unused_imports)]
use std::time::Duration;
use cell::{Cell, Species};


const WIDTH:  u32 = 500;
const HEIGHT: u32 = 350;
const SCALE: u32 = 2;

pub enum ExitCode {
    Success,
    Fail
}

#[derive(Clone, Debug)]
pub enum Error {
    Error(String),
    RenderError(String),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Error(error)
    }
}

pub struct Controls {
    pub mouse_x: i32,
    pub mouse_last_x: i32,
    pub mouse_y: i32,
    pub mouse_last_y: i32,
    pub mouse_pressed_l: bool,
    pub mouse_pressed_r: bool,
    pub pause: bool,
    pub selected_species: Species,
    pub radius: i32,
}

impl Controls {
    pub fn new() -> Self {
        Controls {
            mouse_x: 0,
            mouse_last_x: 0,
            mouse_y: 0,
            mouse_last_y: 0,
            mouse_pressed_l: false,
            mouse_pressed_r: false,
            pause: false,
            selected_species: Species::Sand,
            radius: 10,
        }
    }

    pub fn set_species(&mut self, s: Species) {
        self.selected_species = s;
    }

    pub fn set_radius(&mut self, r: i32) {
        self.radius = r;
    }
}

fn poll_controls(controls: &mut Controls, event_pump: &mut sdl2::EventPump) -> Option<ExitCode> {
    controls.pause = false;
    let mouse_state = event_pump.mouse_state();
    
    controls.mouse_pressed_l = mouse_state
        .is_mouse_button_pressed(sdl2::mouse::MouseButton::Left);
    controls.mouse_pressed_r = mouse_state
        .is_mouse_button_pressed(sdl2::mouse::MouseButton::Right);
    
    controls.mouse_last_x = controls.mouse_x;
    controls.mouse_last_y = controls.mouse_y;
    controls.mouse_x = mouse_state.x() / SCALE as i32;
    controls.mouse_y = mouse_state.y() / SCALE as i32;

    for e in event_pump.poll_iter() {
        match e {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } 
                => return Some(ExitCode::Success),
            Event::KeyDown { keycode, .. } => {
                use Keycode::*;
                if let Some(k) = keycode {
                    controls.set_species(
                        match k {
                            S => Species::Sand,
                            W => Species::Water, 
                            A => Species::Wall,
                            I => Species::Acid,
                            _ => controls.selected_species
                        }
                    );
                    controls.set_radius(
                        match k {
                           Num1 => 1,
                           Num2 => 5, 
                           Num3 => 10,
                           Num4 => 20,
                           Num5 => 30,
                           _ => controls.radius
                        }
                    );
                    match k {
                        P | Space => controls.pause = true,
                        _ => {}
                    }
                }
            }
            _ => return None 
        }
    }
    None
}

fn handle_controls(controls: &Controls, api: &mut api::SandApi) -> Result<(), Error> {
    if controls.mouse_pressed_l {
        for point in util::line(
            controls.mouse_x, 
            controls.mouse_y, 
            controls.mouse_last_x, 
            controls.mouse_last_y) 
        {
            api.brush(
                point.x, 
                point.y, 
                controls.radius, 
                Cell::new(controls.selected_species)
            )?;
        }
    }

    if controls.mouse_pressed_r {
        api.erase(controls.mouse_x, controls.mouse_y, controls.radius)?;
    }

    if controls.pause {
        api.toggle_pause();
    }
    Ok(())
}

fn init_canvas(ctx: &sdl2::Sdl) -> Canvas<Window> {
    let video_subsystem = ctx.video().unwrap(); 
    let window = video_subsystem.window("rust-sdl2 demo", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .build()
        .unwrap(); 
    let mut canvas = window.into_canvas().build().unwrap(); 
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::Add);
    canvas.clear();
    canvas.present();
    canvas
}

pub fn main() -> Result<(), Error>{
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut controls = Controls::new();
    let mut canvas = init_canvas(&sdl_context);
    let mut sand_api = api::SandApi::new();
    'running: loop {
        sdl_context.mouse().show_cursor(false);
        canvas.clear();
        canvas.set_draw_color(Color::BLACK);
        render::render(&mut sand_api, &mut canvas)?;
        render::render_cursor(&controls, &mut canvas)?;
        match poll_controls(&mut controls, &mut event_pump) {
            None => {}
            _    => break 'running
        };
        // The rest of the game loop goes here...
        handle_controls(&mut controls, &mut sand_api)?;
        sand_api.update()?;
        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }
    Ok(())
}
