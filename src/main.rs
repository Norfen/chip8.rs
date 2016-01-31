#![feature(step_by)]
#![feature(plugin)]
#![plugin(docopt_macros)]
#[macro_use]

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate fps_counter;
extern crate read_color;
extern crate rustc_serialize;
extern crate docopt;
extern crate portaudio;

use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
use sdl2_window::Sdl2Window;
use opengl_graphics::{GlGraphics, OpenGL};

use portaudio as pa;

mod chip8;
mod app;

docopt!(Args derive Debug, "
Chip8.

Usage:
	chip8 <filename> [--speed=<hz>] [(--foreground=<color> --background=<color>)] [--no-overdraw]

Options:
    --speed=<hz>           Set the emulation clock speed [default: 240].
	--foreground=<color>   Set the foreground color in hex [default: FFFFFF]
	--background=<color>   Set the background color in hex [default: 000000]
    --no-overdraw          Force a redraw for all DYXN instructions. 
", flag_speed: i32);

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let opengl = OpenGL::V3_2;
    let mut window: Sdl2Window = WindowSettings::new("Chip8.rs", [640, 320])
                                     .opengl(opengl)
                                     .exit_on_esc(true)
                                     .samples(4)
                                     .vsync(true)
                                     .build()
                                     .unwrap();

    let mut pa = pa::PortAudio::new().unwrap();

    let mut app = app::App::init(GlGraphics::new(opengl),
                                 String::from(args.arg_filename.clone()),
                                 if args.flag_speed % 60 == 0 {
                                     args.flag_speed as usize
                                 } else {
                                     panic!("Clock speed {} is not divisible by 60, desync will \
                                             occur.",
                                            args.flag_speed);
                                 },
                                 if let Some((rgb, a)) =
                                        read_color::rgb_maybe_a(&mut args.flag_foreground.chars()) {
                                     [rgb[0],
                                      rgb[1],
                                      rgb[2],
                                      if let Some(alpha) = a {
                                          alpha
                                      } else {
                                          255
                                      }]
                                 } else {
                                     [255, 255, 255, 255]
                                 },
                                 if let Some((rgb, a)) =
                                        read_color::rgb_maybe_a(&mut args.flag_background.chars()) {
                                     [rgb[0],
                                      rgb[1],
                                      rgb[2],
                                      if let Some(alpha) = a {
                                          alpha
                                      } else {
                                          255
                                      }]
                                 } else {
                                     [0, 0, 0, 255]
                                 },
                                 args.flag_no_overdraw,
                                 &pa);

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(k) = e.press_args() {
            app.keypress(&k);
        }

        if let Some(u) = e.release_args() {
            app.unkeypress(&u);
        }
    }
}
