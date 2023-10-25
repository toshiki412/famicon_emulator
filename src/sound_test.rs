use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct TriangleNote {
    hz: f32,
}
struct TriangleWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<TriangleNote>,
    note: TriangleNote,
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //三角波の生成
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(note) => self.note = note,
                Err(_) => {}
            }

            *x = (if self.phase <= 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } - 0.5)
                * 2.0;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

const base: f32 = 1789772.5;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<TriangleNote>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| TriangleWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            note: TriangleNote { hz: 440.0 },
        })
        .unwrap();

    device.resume();
    std::thread::sleep(Duration::from_millis(2000));
}
