//モジュールのインポートはメインに書かなきゃいけない
mod apu;
mod bus;
mod cartrige;
mod cpu;
mod frame;
mod joypad;
mod mapper;
mod opscodes;
mod palette;
mod ppu;
mod render;
mod rom;

use crate::cpu::IN_TRACE;
use crate::rom::Rom;

use log::{debug, info, trace};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

// use crate::mapper::Mapper1;
use crate::mapper::create_mapper;
use crate::mapper::Mapper;

use self::bus::{Bus, Mem};
use self::cpu::CPU;
use apu::NesAPU;
use cartrige::load_rom;
use joypad::Joypad;

use frame::Frame;
use ppu::NesPPU;
// initialize SDL
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

static mut MAPPER: Lazy<Box<dyn Mapper>> = Lazy::new(|| create_mapper(Rom::empty()));

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let style = buf.style();
            if unsafe { IN_TRACE } {
                writeln!(buf, "[TRACE] {}", style.value(record.args()))
            } else {
                writeln!(buf, "        {}", style.value(record.args()))
            }
        })
        .format_timestamp(None)
        .init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES Emulator", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // canvas.set_scale(10.0, 10.0).unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        // .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    //mapper0
    // let rom = load_rom("rom/mario.nes");
    //mapper1
    let rom = load_rom("rom/dragon_quest4.nes");
    //mapper2
    // let rom = load_rom("rom/dragon_quest2.nes");
    //mapper3
    // let rom = load_rom("rom/dragon_quest1.nes");
    //mapper4
    // let rom = load_rom("rom/finalfantasy3.nes");

    info!(
        "ROM: mapper={}, mirroring={:?}, chr_ram={}",
        rom.mapper, rom.screen_mirroring, rom.is_chr_ram
    );

    unsafe {
        *MAPPER = create_mapper(rom);
    }

    let mut now = Instant::now();
    let interval = 1000 * 1000 * 1000 / 60; //60fps per frame

    let apu = NesAPU::new(&sdl_context);
    let bus = Bus::new(
        apu,
        move |ppu: &NesPPU, joypad1: &mut Joypad, frame: &Frame| {
            //frameのデータをテクスチャに更新します。このテクスチャはゲーム画面を表現します。
            texture.update(None, &frame.data, 256 * 3).unwrap();

            //テクスチャをウィンドウのキャンバスにコピーします。
            canvas.copy(&texture, None, None).unwrap();

            //ウィンドウ上にゲーム画面を表示します。
            canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),

                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            joypad1.set_button_pressed_status(*key, true);
                        }
                    }

                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            joypad1.set_button_pressed_status(*key, false);
                        }
                    }
                    _ => { /* do nothing */ }
                }
            }
            let time = now.elapsed().as_nanos();
            if time < interval {
                sleep(Duration::from_nanos((interval - time) as u64));
            }
            now = Instant::now();
        },
    );

    //CPU構造体の新しいインスタンスを作成し、busを渡します。
    // これにより、CPUがエミュレーションされ、NESのプログラムを実行できます。
    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.run_with_callback(move |_| {});
    // cpu.run_with_callback(move |cpu| {
    //     if log_enabled!(Level::Trace) {
    //         trace(cpu);
    //     }
    // });
}

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                std::process::exit(0);
            }

            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                cpu.mem_write(0xff, 0x77); //0xffをユーザが最後に押したボタンのコード
            }

            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                cpu.mem_write(0xff, 0x73);
            }

            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                cpu.mem_write(0xff, 0x61);
            }

            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                cpu.mem_write(0xff, 0x64);
            }

            _ => { /*do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GRAY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

//画面の情報が変わったときだけ画面の更新をする
//毎回画面の更新処理をするのは重いため
//frameが前回のframe
fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;

    for i in 0x0200..0x0600 {
        //0200~0600を画面のメモリ領域としている
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}
