use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use crate::MAPPER;

use super::{ChannelEvent, NES_CPU_CLOCK};

static DMC_FREQUENCY_TABLE: [u16; 16] = [
    0x01AC, 0x017C, 0x0154, 0x0140, 0x011E, 0x00FE, 0x00E2, 0x00D6, 0x00BE, 0x00A0, 0x008E, 0x0080,
    0x006A, 0x0054, 0x0048, 0x0036,
];

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
    pub sample_byte_count: u8, //サンプル長
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
    IrqEnable(bool),
    Loop(bool),
    Frequency(u8),
    Delta(u8),
    SampleStartAddr(u8),
    SampleByteCount(u8),

    Enable(bool),
    Reset(),
}

pub struct DMCWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<DMCEvent>,
    sender: Sender<ChannelEvent>,
    enabled_sound: bool,

    irq_enable: bool,
    loop_flag: bool,
    frequency_index: u8,
    delta_counter: u8,
    sample_start_addr: u8,
    sample_byte_count: u8,

    data: u8,
    org_freq: f32,
    org_sample_start_addr: u16,
    org_sample_byte_count: u32,
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
                    Ok(DMCEvent::IrqEnable(b)) => self.irq_enable = b,
                    Ok(DMCEvent::Loop(b)) => self.loop_flag = b,
                    Ok(DMCEvent::Frequency(f)) => {
                        self.frequency_index = f;
                        self.org_freq = NES_CPU_CLOCK / (DMC_FREQUENCY_TABLE[f as usize] as f32)
                    }
                    Ok(DMCEvent::Delta(d)) => {
                        self.delta_counter = d;
                        self.sample_byte_count = 1;
                        self.org_sample_byte_count = (1 * 8) as u32 * 0x10 + 1;
                    }
                    Ok(DMCEvent::SampleStartAddr(sa)) => {
                        self.sample_start_addr = sa;
                        self.org_sample_start_addr = sa as u16 * 0x40 + 0xC000;
                    }
                    Ok(DMCEvent::SampleByteCount(bc)) => {
                        self.sample_byte_count = bc;
                        self.org_sample_byte_count = (bc * 8) as u32 * 0x10 + 1;
                    }
                    Ok(DMCEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(DMCEvent::Reset()) => {}
                    Err(_) => break,
                }
            }

            let last_phase = self.phase;
            self.phase = (self.phase + self.org_freq / self.freq) % 1.0;

            if last_phase > self.phase {
                if self.org_sample_byte_count == 0 {
                    *x = 0.0;
                    continue;
                }
                if self.org_sample_byte_count & 0x0007 == 0 {
                    if self.org_sample_byte_count != 0 {
                        unsafe {
                            self.data = MAPPER.read_prg_rom(self.org_sample_start_addr);
                        };
                        if self.org_sample_start_addr == 0xFFFF {
                            self.org_sample_start_addr = 0x8000;
                        } else {
                            self.org_sample_start_addr += 1;
                        }
                    }
                }

                if self.org_sample_byte_count != 0 {
                    if self.data & 0x01 == 0x00 {
                        if self.delta_counter > 1 {
                            self.delta_counter -= 2
                        }
                    } else {
                        if self.delta_counter < 126 {
                            self.delta_counter += 2
                        }
                    }
                    self.data = self.data >> 1;
                    self.org_sample_byte_count -= 1;
                    self.sender
                        .send(ChannelEvent::LengthCounter(self.org_sample_byte_count))
                        .unwrap();
                }

                if self.org_sample_byte_count == 0 {
                    if self.loop_flag {
                        self.set_delta();
                    } else {
                        if self.irq_enable {
                            // TODO IRQを発生させる
                        }
                    }
                }
            }
            if self.delta_counter == 0 || self.org_sample_byte_count == 0 {
                *x = 0.0;
            } else {
                *x = (self.delta_counter as f32 - 64.0) / 64.0;
            }

            if !self.enabled_sound {
                *x = 0.0;
            }
        }
    }
}

impl DMCWave {
    fn set_delta(&mut self) {
        self.delta_counter = self.delta_counter;
        self.org_sample_start_addr = self.sample_start_addr as u16 * 0x40 + 0xC000;
        self.org_sample_byte_count = (self.sample_byte_count * 8) as u32 * 0x10 + 1;
        self.data = 0;
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
            irq_enable: false,
            loop_flag: false,
            frequency_index: 0,
            delta_counter: 0,
            sample_start_addr: 0,
            sample_byte_count: 1, // (0*8 * 0x10 + 1)
            data: 0,
            org_freq: NES_CPU_CLOCK / DMC_FREQUENCY_TABLE[0] as f32,
            org_sample_start_addr: 0xC000, // 0*0x40 + 0xC000
            org_sample_byte_count: 1,      // (0*8 * 0x10 + 1)
        })
        .unwrap();

    device.resume();
    (device, sender, receiver2)
}
