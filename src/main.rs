extern crate sdl2; 
pub mod cell;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

const WIDTH:  u32 = 640;
const HEIGHT: u32 = 480; 

pub enum ExitCode {
    Success,
    Fail
}

pub fn poll_for_exit(ctx: &sdl2::Sdl) -> Option<ExitCode> {
    let mut event_pump = ctx.event_pump().unwrap();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } 
                => return Some(ExitCode::Success),
            _   => return None
        }
    }
    None
}

pub fn init_canvas(ctx: &sdl2::Sdl) -> Canvas<Window> {
    let video_subsystem = ctx.video().unwrap(); 
    let window = video_subsystem.window("rust-sdl2 demo", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap(); 
    let mut canvas = window.into_canvas().build().unwrap(); 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    canvas
}

pub fn main() -> Result<(), cell::Error>{
    let sdl_context = sdl2::init().unwrap();
    let mut canvas = init_canvas(&sdl_context);
    let mut sand_api = cell::SandApi::new();
    'running: loop {
        canvas.clear();
        let dots = sand_api.get_dots();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.draw_points(dots.as_slice()).unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        match poll_for_exit(&sdl_context) {
            None => {}
            _    => break 'running
        };
        // The rest of the game loop goes here...
        sand_api.update()?;
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
