use std::fs::File;
use std::io::Read;
use std::time::Duration;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod chip8;
use chip8::Chip8;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0
        }
    }
}

fn main() -> Result<(), String> {
    let width = 800;
    let height = 600;

    let sdl_context = sdl2::init()?;

    // Audio
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.00005,
        })
        .unwrap();

    device.resume();

    // Display
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("CHIP-8 RS", width, height)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let screen_area = Rect::new(0, 0, width, height);
    let clear_color = Color::RGB(0, 0, 0);

    let mut running = true;
    let mut event_queue = sdl_context.event_pump().unwrap();

    let mut i = 0;

    let mut chip8 = Chip8::new();
    while running {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();

        chip8.run();

        for event in event_queue.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    running = false;
                }
                _ => {}
            }
        }
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}

// fn main() {
//     let mut f = std::fs::read("src/IBM_logo.ch8").unwrap();
//     println!("{:?}", f);
//     let mut chip8 = Chip8::new();
// }
