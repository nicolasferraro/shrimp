use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::joypad::Joypad;
use crate::ppu::PPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, TextureAccess};
use sdl2::{pixels::PixelFormatEnum, video::Window};
use std::cell::RefCell;
use std::rc::Rc;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

pub struct NES {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
    scale: u8,
}

impl NES {
    pub fn new(opts: super::Options) -> Self {
        let cartridge = Cartridge::from_path(opts.rom.as_str()).unwrap();
        let cartridge = Rc::new(RefCell::new(cartridge));

        let ppu = PPU::new(cartridge.clone());
        let ppu = Rc::new(RefCell::new(ppu));

        let cpu = CPU::new(cartridge, ppu.clone());
        Self {
            cpu,
            ppu,
            scale: opts.scale,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem: sdl2::VideoSubsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "NEMU",
                SCREEN_WIDTH as u32 * self.scale as u32,
                SCREEN_HEIGHT as u32 * self.scale as u32,
            )
            .opengl()
            .build()?;

        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

        let mut event_pump = sdl_context.event_pump()?;
        let mut canvas: Canvas<Window> = window.into_canvas().accelerated().build()?;

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::BGR24,
            TextureAccess::Streaming,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        'running: loop {
            self.cpu.tick();
            let mut ppu = self.ppu.borrow_mut();
            ppu.tick(&mut self.cpu);

            if ppu.frame_complete {
                texture.update(None, &ppu.screen, SCREEN_WIDTH * 3)?;

                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();

                while let Some(event) = event_pump.poll_event() {
                    let c = &mut self.cpu.joypad_1;
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        Event::KeyUp {
                            keycode: Some(keycode),
                            ..
                        } => set_keys(c, keycode, false),
                        Event::KeyDown {
                            keycode: Some(keycode),
                            ..
                        } => set_keys(c, keycode, true),
                        _ => {}
                    }
                }

                // 60 FPS
                std::thread::sleep(std::time::Duration::from_nanos(16000000));
            }
        }

        Ok(())
    }
}

fn set_keys(c: &mut Joypad, keycode: Keycode, pressed: bool) {
    match keycode {
        Keycode::Return => c.start = pressed,
        Keycode::LShift => c.select = pressed,
        Keycode::X => c.a = pressed,
        Keycode::Z => c.b = pressed,
        Keycode::Up => c.up = pressed,
        Keycode::Down => c.down = pressed,
        Keycode::Left => c.left = pressed,
        Keycode::Right => c.right = pressed,
        _ => {}
    }
}
