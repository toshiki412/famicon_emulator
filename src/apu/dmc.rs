use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use super::ChannelEvent;

pub struct DMCRegister {
    //4010
    pub irq_enable: bool,
    pub loop_flag: bool,
    pub frequency_index: u8,

    //4011
    pub delta_counter: u8,

    //4012
    pub sample_start_addr: u8,

    //4013
    pub sample_byte_count: u8,
}

impl DMCRegister {
    pub fn new() -> Self {
        DMCRegister {
            irq_enable: false,
            loop_flag: false,
            frequency_index: 0,
            delta_counter: 0,
            sample_start_addr: 0,
            sample_byte_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4010 => {
                self.irq_enable = value & 0x80 != 0;
                self.loop_flag = value & 0x40 != 0;
                self.frequency_index = value & 0x0F;
            }
            0x4011 => self.delta_counter = value & 0x7F,
            0x4012 => self.sample_start_addr = value,

            0x4013 => self.sample_byte_count = value,
            _ => panic!("cant be"),
        }
    }
}

pub enum DMCEvent {
    Enable(bool),
    Reset(),
}

pub struct DMCWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<DMCEvent>,
    sender: Sender<ChannelEvent>,
    enabled_sound: bool,
}

impl AudioCallback for DMCWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //DMCの音の生成
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(DMCEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(DMCEvent::Reset()) => {}
                    Err(_) => break,
                }
            }

            *x = 0.0;
        }
    }
}

//sdl周りの初期化
pub fn init_DMC(
    sdl_context: &sdl2::Sdl,
) -> (
    AudioDevice<DMCWave>,
    Sender<DMCEvent>,
    Receiver<ChannelEvent>,
) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<DMCEvent>();
    let (sender2, receiver2) = channel::<ChannelEvent>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| DMCWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            enabled_sound: true,
        })
        .unwrap();

    device.resume();
    (device, sender, receiver2)
}
