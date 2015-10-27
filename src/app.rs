use fps_counter::FPSCounter;
use graphics::*;
use opengl_graphics::{GlGraphics, Texture};
use piston::input::*;
use sound_stream::{CallbackFlags, CallbackResult, SoundStream, Settings, StreamParams};
use sound_stream::output::NonBlockingStream;

use chip8::Chip8;

type RGBA = [f32; 4];

trait RGBATrait {
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> RGBA;
    fn rgb(r: f32, g: f32, b: f32) -> RGBA;
    fn from_u8(xs: [u8; 4]) -> RGBA;
}

impl RGBATrait for RGBA {
    #[inline]
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        [r, g, b, a]
    }

    #[inline]
    fn rgb(r: f32, g: f32, b: f32) -> RGBA {
        [r, g, b, 1.0]
    }

    #[inline]
    fn from_u8(xs: [u8; 4]) -> RGBA {
        [xs[0] as f32 / 255.0, xs[1] as f32 / 255.0, xs[2] as f32 / 255.0, xs[3] as f32 / 255.0]
    }
}

#[inline]
fn square_wave(phase: f64) -> f32 {
    ((phase * ::std::f64::consts::PI * 2.0).sin().signum() * 0.25) as f32
}

pub struct App {
    gl: GlGraphics,
    c8: Chip8,
    ticker: f64,
    fps_counter: FPSCounter,
    clock_counter: FPSCounter,
    lastfps: usize,
    lasthz: usize,
    clockspeed: i32,
    background_color: RGBA,
    foreground_color: RGBA,
    sound: Option<NonBlockingStream>,
}

impl App {
    pub fn init(gl: GlGraphics, program_file: String, clock: i32, foreground: [u8; 4], background: [u8; 4], no_overdraw: bool) -> App {
        let mut temp = App {
            gl: gl,
            c8: Chip8::init(),
            ticker: 0.0,
            fps_counter: FPSCounter::new(),
            clock_counter: FPSCounter::new(),
            lastfps: 0,
            lasthz: 0,
            clockspeed: clock,
            foreground_color: RGBA::from_u8(foreground),
            background_color: RGBA::from_u8(background),
            sound: None,
        };
        temp.c8.load_program(program_file);
        temp.c8.no_overdraw = no_overdraw;
        temp
    }

    #[allow(dead_code)]
    pub fn reload(&mut self, filename: String) {
        self.c8 = Chip8::init();
        self.c8.load_program(filename);
    }

    pub fn render(&mut self, args: &RenderArgs) {
        if self.c8.draw_flag {
            use graphics::*;

            let (memwidth, memheight) = self.c8.screen_dimens();
            let wscale = args.width as f64 / memwidth as f64;
            let hscale = args.height as f64 / memheight as f64;
            let fcolor = self.foreground_color;
            let bcolor = self.background_color;
            let texture = Texture::from_memory_alpha(&self.c8.gfx,
                                                     memwidth as u32,
                                                     memheight as u32)
                              .unwrap();

            self.gl.draw(args.viewport(), |c, gl| {
                clear(bcolor, gl);
                Image::new_color(fcolor).draw(&texture,
                                              default_draw_state(),
                                              c.transform.scale(wscale, hscale),
                                              gl);
            });

            self.c8.draw_flag = false;
        }
        self.lastfps = self.fps_counter.tick();
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.ticker += args.dt;
        while self.ticker > 1.0 / 60.0 {
            self.c8.tick();
            self.ticker -= 1.0 / 60.0;
        }
        if self.c8.sound_timer > 0 && self.sound.is_none() {
            let mut timer = self.c8.sound_timer as f64 * (1.0 / 60.0);
            let mut phase = 0.0;
            self.sound = Some(SoundStream::new()
                                  .output(StreamParams::new())
                                  .run_callback(Box::new(move |output: &mut [f32],
                                                               settings: Settings,
                                                               dt: f64,
                                                               _: CallbackFlags| {
                                      for frame in output.chunks_mut(settings.channels as usize) {
                                          let snd = square_wave(phase);
                                          for channel in frame {
                                              *channel = snd;
                                          }
                                          phase += 200.0 / settings.sample_hz as f64;
                                      }
                                      timer -= dt;
                                      if timer >= 0.0 {
                                          CallbackResult::Continue
                                      } else {
                                          CallbackResult::Complete
                                      }
                                  }))
                                  .unwrap());
        }
        if self.sound.as_ref().map_or(false, |s| s.is_active().as_ref().ok() == Some(&false)) {
            self.sound = None;
        }
        for _ in 0..(((self.clockspeed as f64) * args.dt).round() as usize) {
            self.c8.step();
            self.lasthz = self.clock_counter.tick();
        }
        if ((self.lasthz as f64) * 0.05) + (self.lasthz as f64) < (self.clockspeed
        as f64) || (self.lasthz as f64) - ((self.lasthz as f64) * 0.05) >
        (self.clockspeed as f64) {
        	println!("CPU is out of sync: {}Hz", self.lasthz);
        }
    }

    #[inline]
    pub fn keypress(&mut self, args: &Button) {
        self.handle_keys(args, true);
    }

    #[inline]
    pub fn unkeypress(&mut self, args: &Button) {
        self.handle_keys(args, false);
    }

    fn handle_keys(&mut self, key: &Button, pressed: bool) {
        use piston::input::Button::Keyboard;
        match *key {
            Keyboard(Key::D1) => self.c8.update_keys(1, pressed),
            Keyboard(Key::D2) => self.c8.update_keys(2, pressed),
            Keyboard(Key::D3) => self.c8.update_keys(3, pressed),
            Keyboard(Key::D4) => self.c8.update_keys(0xC, pressed),
            Keyboard(Key::Q) => self.c8.update_keys(4, pressed),
            Keyboard(Key::W) => self.c8.update_keys(5, pressed),
            Keyboard(Key::E) => self.c8.update_keys(6, pressed),
            Keyboard(Key::R) => self.c8.update_keys(0xD, pressed),
            Keyboard(Key::A) => self.c8.update_keys(7, pressed),
            Keyboard(Key::S) => self.c8.update_keys(8, pressed),
            Keyboard(Key::D) => self.c8.update_keys(9, pressed),
            Keyboard(Key::F) => self.c8.update_keys(0xE, pressed),
            Keyboard(Key::Z) => self.c8.update_keys(0xA, pressed),
            Keyboard(Key::X) => self.c8.update_keys(0, pressed),
            Keyboard(Key::C) => self.c8.update_keys(0xB, pressed),
            Keyboard(Key::V) => self.c8.update_keys(0xF, pressed),
            _ => {}
        }
    }
}
