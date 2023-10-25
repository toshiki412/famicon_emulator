use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseNote {
    hz: f32,
    is_long: bool,
    volume: f32,
}
struct NoiseWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<NoiseNote>,
    is_sound: bool,
    long_random: NoiseRandom,
    short_random: NoiseRandom,

    note: NoiseNote,
}

impl AudioCallback for NoiseWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //ノイズの生成
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(note) => self.note = note,
                Err(_) => {}
            }

            *x = if self.is_sound { 0.0 } else { 1.0 } * self.note.volume;
            let last_phase = self.phase;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
            if last_phase > self.phase {
                self.is_sound = if self.note.is_long {
                    self.long_random.next()
                } else {
                    self.short_random.next()
                };
            }
        }
    }
}

// ノイズ
struct NoiseRandom {
    bit: u8,
    value: u16,
}

impl NoiseRandom {
    pub fn new_long() -> Self {
        NoiseRandom { bit: 1, value: 1 }
    }

    pub fn new_short() -> Self {
        NoiseRandom { bit: 6, value: 1 }
    }

    pub fn next(&mut self) -> bool {
        //ロングモード時はビット0とビット1のXORを入れる
        let b = (self.value & 0x01) ^ ((self.value >> self.bit) & 0x01);
        self.value = self.value >> 1;
        self.value = self.value & 0b011_1111_1111_1111 | b << 14;

        //シフトレジスタのビット0が1なら出力は0
        self.value & 0x01 != 0
    }
}

const base: f32 = 1789772.5;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<NoiseNote>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| NoiseWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            is_sound: false,
            note: NoiseNote {
                hz: base / 0x7f as f32, //レジスタに9を書き込むときのノイズ
                volume: 0.1,
                is_long: true,
            },
            long_random: NoiseRandom::new_long(),
            short_random: NoiseRandom::new_short(),
        })
        .unwrap();

    device.resume();
    std::thread::sleep(Duration::from_millis(2000));

    sender
        .send(NoiseNote {
            hz: base / 0x20 as f32,
            is_long: false,
            volume: 0.1,
        })
        .unwrap();
    std::thread::sleep(Duration::from_millis(2000));
}
