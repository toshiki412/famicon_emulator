use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct SquareNote {
    hz: f32,
    volume: f32,
    duty: f32, //波の上と下の比率
}
struct SquareWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<SquareNote>,
    note: SquareNote,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //矩形波の生成
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(note) => self.note = note,
                Err(_) => {}
            }

            *x = if self.phase <= self.note.duty {
                self.note.volume
            } else {
                -self.note.volume
            };
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

const la4_fra: f32 = 415.305;
const shi4_fra: f32 = 466.164;
const do5: f32 = 523.251;
const re5: f32 = 587.330;
const mi5_fra: f32 = 622.254;
const fa5: f32 = 698.456;
const so5: f32 = 783.991;
const so5_fra: f32 = 739.989;
const la5_fra: f32 = 830.609;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<SquareNote>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| SquareWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            note: SquareNote {
                hz: 0.0,
                volume: 0.0,
                duty: 0.0,
            },
        })
        .unwrap();

    device.resume();

    // ありったけの
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 100);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 300);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 100);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 800);
    mute(sender.clone(), 30);

    // 夢を
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 300);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 800);
    mute(sender.clone(), 200);

    //かき集め //
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), la5_fra, 0.1, 0.5, 300);
    play_sound(sender.clone(), so5, 0.1, 0.5, 400);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 1500);
    mute(sender.clone(), 300);

    // 探し物を
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 100);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 300);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 100);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 800);
    mute(sender.clone(), 30);

    // 探し
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 300);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 500);
    mute(sender.clone(), 30);

    //に行くのさ
    play_sound(sender.clone(), fa5, 0.1, 0.5, 400);
    play_sound(sender.clone(), la5_fra, 0.1, 0.5, 400);
    play_sound(sender.clone(), so5, 0.1, 0.5, 400);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 1200);
    mute(sender.clone(), 600);

    // ポケットのコイン
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 400);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 500);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), la4_fra, 0.1, 0.5, 500);
    play_sound(sender.clone(), so5, 0.1, 0.5, 200);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 800);
    mute(sender.clone(), 100);

    // それとyou wanna be my friend
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    mute(sender.clone(), 20);
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    mute(sender.clone(), 20);
    play_sound(sender.clone(), re5, 0.1, 0.5, 200);
    play_sound(sender.clone(), do5, 0.1, 0.5, 400);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 400);
    play_sound(sender.clone(), do5, 0.1, 0.5, 1500);
    mute(sender.clone(), 300);

    //ウィーアー、ウィーアー、オンザクルーズ
    play_sound(sender.clone(), la4_fra, 0.1, 0.5, 600);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 800);
    mute(sender.clone(), 200);
    play_sound(sender.clone(), shi4_fra, 0.1, 0.5, 600);
    mute(sender.clone(), 30);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 600);
    mute(sender.clone(), 70);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), so5_fra, 0.1, 0.5, 1700);
    mute(sender.clone(), 500);

    // ウィーアー
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 200);
    play_sound(sender.clone(), fa5, 0.1, 0.5, 200);
    play_sound(sender.clone(), mi5_fra, 0.1, 0.5, 400);
}

pub fn play_sound(sender: Sender<SquareNote>, hz: f32, volume: f32, duty: f32, msec: u64) {
    sender
        .send(SquareNote {
            hz: hz,
            volume: volume,
            duty: duty,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(msec));
}

pub fn mute(sender: Sender<SquareNote>, msec: u64) {
    sender
        .send(SquareNote {
            hz: 0.0,
            volume: 0.0,
            duty: 0.0,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(msec));
}
