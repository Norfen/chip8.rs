use fps_counter::FPSCounter;
use graphics::*;
use opengl_graphics::GlGraphics;
use piston::input::*;

use chip8::Chip8;

type RGBA = [f32; 4];

trait RGBATrait {
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> RGBA;
    fn rgb(r: f32, g: f32, b: f32) -> RGBA;
    fn from_u8(xs: [u8; 4]) -> RGBA;
}

impl RGBATrait for RGBA {
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        [r, g, b, a]
    }

    fn rgb(r: f32, g: f32, b: f32) -> RGBA {
        [r, g, b, 1.0]
    }

    fn from_u8(xs: [u8; 4]) -> RGBA {
        [xs[0] as f32 / 255.0, xs[1] as f32 / 255.0, xs[2] as f32 / 255.0, xs[3] as f32 / 255.0]
    }
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
}

impl App {
    pub fn init(gl: GlGraphics,
                program_file: String,
                clock: i32,
                foreground: [u8; 4],
                background: [u8; 4])
                -> App {
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
        };
        temp.c8.load_program(program_file);
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

            let w = args.width;

            let gfxbuffer = &self.c8.gfx;
            let (memwidth, memheight) = self.c8.screen_dimens();
            let pixelsize = (w as f64) / (memwidth as f64);         

            let foreground = self.foreground_color;
            let background = self.background_color;
            let pixel = rectangle::square(0.0, 0.0, pixelsize);

            self.gl.draw(args.viewport(), |c, gl| {
                clear(background, gl);

                for y in 0..memheight {
                    for x in 0..memwidth {
                        rectangle(if gfxbuffer[((y * memwidth) + x) as usize] {
                                      foreground
                                  } else {
                                      background
                                  },
                                  pixel,
                                  c.transform.trans((x as f64) * pixelsize, (y as f64) * pixelsize),
                                  gl)
                    }
                }
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
        for _ in 0..(((self.clockspeed as f64) * args.dt).round() as usize) {
            self.c8.step();
            self.lasthz = self.clock_counter.tick();
        }
        // if ((self.lasthz as f64) * 0.05) + (self.lasthz as f64) < (self.clockspeed as f64) || (self.lasthz as f64) - ((self.lasthz as f64) * 0.05) > (self.clockspeed as f64) {
        // 	println!("CPU is out of sync: {}Hz", self.lasthz);
        // }
    }

    pub fn keypress(&mut self, args: &Button) {
        self.handle_keys(args, true);
    }

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
