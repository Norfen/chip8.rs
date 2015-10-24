#![feature(step_by)]
#![feature(wrapping)]
#![feature(plugin)]
// #![plugin(docopt_macros)]

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate fps_counter;
extern crate read_color;
extern crate rustc_serialize;
extern crate docopt;

use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
use sdl2_window::Sdl2Window;
use opengl_graphics::{GlGraphics, OpenGL};
use docopt::Docopt;

mod chip8;
mod app;

const USAGE: &'static str = "
Chip8.

Usage:
	chip8 <filename> [--speed=<hz>] \
                             [(--foreground=<color> --background=<color>)]

Options:
	\
                             --speed=<hz>
	--foreground=<color>
	--background=<color>
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_filename: String,
    flag_speed: Option<i32>,
    flag_foreground: Option<String>,
    flag_background: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let opengl = OpenGL::V3_2;
    let window: Sdl2Window = WindowSettings::new("Chip8.rs", [640, 320])
                                 .opengl(opengl)
                                 .exit_on_esc(true)
                                 .samples(4)
                                 .vsync(true)
                                 .build()
                                 .unwrap();

    let mut app = app::App::init(GlGraphics::new(opengl),
                                 String::from(args.arg_filename.clone()),
                                 match args.flag_speed {
                                     Some(clock) => {
                                     	if clock % 60 == 0 {
                                     		clock
                                     	} else {
                                     		panic!("Clock speed {} is not divisible by 60, desync will occur.", clock);
                                     	}
                                     },
                                     None => 120,
                                 },
                                 match args.flag_foreground {
                                     Some(color) => {
                                         if let Some((rgb, a)) =
                                                read_color::rgb_maybe_a(&mut color.chars()) {
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
                                         }
                                     }
                                     None => [255, 255, 255, 255],
                                 },
                                 match args.flag_background {
                                     Some(color) => {
                                         if let Some((rgb, a)) =
                                                read_color::rgb_maybe_a(&mut color.chars()) {
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
                                         }
                                     }
                                     None => [0, 0, 0, 255],
                                 });

    for e in window.events() {
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
