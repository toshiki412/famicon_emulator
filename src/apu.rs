use bitflags::{bitflags, Flags};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub struct NesAPU {
    ch1_register: Ch1Register,
    ch2_register: Ch2Register,
    ch3_register: Ch3Register,
    ch4_register: Ch4Register,

    frame_counter: FrameCounter,
    status: StatusRegister,
    cycles: usize,
    counter: usize,

    ch1_device: AudioDevice<SquareWave>,
    ch2_device: AudioDevice<SquareWave>,
    ch3_device: AudioDevice<TriangleWave>,
    ch4_device: AudioDevice<NoiseWave>,

    ch1_sender: Sender<SquareEvent>,
    ch2_sender: Sender<SquareEvent>,
    ch3_sender: Sender<TriangleEvent>,
    ch4_sender: Sender<NoiseEvent>,
}

const NES_CPU_CLOCK: f32 = 1_789_772.; //1.78MHz

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender) = init_square(&sdl_context);
        let (ch2_device, ch2_sender) = init_square(&sdl_context);
        let (ch3_device, ch3_sender) = init_triangle(&sdl_context);
        let (ch4_device, ch4_sender) = init_noise(&sdl_context);
        NesAPU {
            ch1_register: Ch1Register::new(),
            ch2_register: Ch2Register::new(),
            ch3_register: Ch3Register::new(),
            ch4_register: Ch4Register::new(),

            frame_counter: FrameCounter::new(),
            status: StatusRegister::new(),
            cycles: 0,
            counter: 0,

            ch1_device: ch1_device,
            ch2_device: ch2_device,
            ch3_device: ch3_device,
            ch4_device: ch4_device,

            ch1_sender: ch1_sender,
            ch2_sender: ch2_sender,
            ch3_sender: ch3_sender,
            ch4_sender: ch4_sender,
        }
    }

    pub fn write_1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);

        let duty = match self.ch1_register.duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.5,
            0x03 => 0.75,
            _ => panic!("cant be"),
        };

        let hz = NES_CPU_CLOCK / (16.0 * (self.ch1_register.frequency as f32 + 1.0));

        //sdlに送る
        self.ch1_sender
            .send(SquareEvent::Note(SquareNote { hz: hz, duty: duty }))
            .unwrap();

        self.ch1_sender
            .send(SquareEvent::Envelope(Envelope::new(
                self.ch1_register.volume,
                self.ch1_register.envelope_flag,
                !self.ch1_register.key_off_counter_flag,
            )))
            .unwrap();
    }

    pub fn write_2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);

        let duty = match self.ch2_register.duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.5,
            0x03 => 0.75,
            _ => panic!("cant be"),
        };

        let hz = NES_CPU_CLOCK / (16.0 * (self.ch2_register.frequency as f32 + 1.0));
        //sdlに送る
        self.ch2_sender
            .send(SquareEvent::Note(SquareNote { hz: hz, duty: duty }))
            .unwrap();

        self.ch2_sender
            .send(SquareEvent::Envelope(Envelope::new(
                self.ch2_register.volume,
                self.ch2_register.envelope_flag,
                !self.ch2_register.key_off_counter_flag,
            )))
            .unwrap();
    }

    pub fn write_3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);

        let hz = NES_CPU_CLOCK / (32.0 * (self.ch2_register.frequency as f32 + 1.0));
        //sdlに送る
        self.ch3_sender
            .send(TriangleEvent::Note(TriangleNote { hz: hz }))
            .unwrap();
    }

    pub fn write_4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);

        let is_long = match self.ch4_register.kind {
            NoiseKind::Long => true,
            _ => false,
        };

        let volume = (self.ch4_register.volume as f32) / 15.0;

        let hz = NES_CPU_CLOCK / NOISE_TABLE[self.ch4_register.noise_hz as usize] as f32;

        //sdlに送る
        self.ch4_sender
            .send(NoiseEvent::Note(NoiseNote {
                hz: hz,
                volume: volume,
                is_long: is_long,
            }))
            .unwrap();
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter.update(value);

        //4017への書き込みによって分周期とシーケンサをリセットする
        self.cycles = 0;
        self.counter = 0;
    }

    pub fn read_status(&mut self) -> u8 {
        let res = self.status.bits();
        self.status.remove(StatusRegister::ENABLE_FRAME_IRQ);
        res
    }

    pub fn write_status(&mut self, data: u8) {
        self.status.update(data);

        self.ch1_sender
            .send(SquareEvent::Enable(
                self.status.contains(StatusRegister::ENABLE_1CH),
            ))
            .unwrap();

        self.ch2_sender
            .send(SquareEvent::Enable(
                self.status.contains(StatusRegister::ENABLE_2CH),
            ))
            .unwrap();

        self.ch3_sender
            .send(TriangleEvent::Enable(
                self.status.contains(StatusRegister::ENABLE_3CH),
            ))
            .unwrap();

        self.ch4_sender
            .send(NoiseEvent::Enable(
                self.status.contains(StatusRegister::ENABLE_4CH),
            ))
            .unwrap();
    }

    pub fn irq(&self) -> bool {
        self.status.contains(StatusRegister::ENABLE_FRAME_IRQ)
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        //一周期分
        let interval = 7457;

        //7457サイクルごとにここが呼び出される
        if self.cycles >= interval {
            self.cycles -= interval;
            self.counter += 1;

            match self.frame_counter.mode() {
                4 => {
                    if self.counter == 2 || self.counter == 4 {
                        //長さカウンタとスイープユニットのクロック生成
                    }
                    if self.counter == 4 {
                        //割り込みフラグセット
                        self.counter = 0;
                        self.status.insert(StatusRegister::ENABLE_FRAME_IRQ);
                    }
                    //エンベロープと三角波の線形カウンタのクロックサイクル
                    self.ch1_sender.send(SquareEvent::EnvelopeTick()).unwrap();
                    self.ch2_sender.send(SquareEvent::EnvelopeTick()).unwrap();
                    self.ch4_sender.send(NoiseEvent::EnvelopeTick()).unwrap();
                }
                5 => {
                    if self.counter == 0 || self.counter == 2 {
                        // 長さカウンタとスイープユニットのクロック生成
                    }
                    if self.counter >= 4 {
                        // エンベロープと三角波の線形カウンタのクロック生成
                        self.ch1_sender.send(SquareEvent::EnvelopeTick()).unwrap();
                        self.ch2_sender.send(SquareEvent::EnvelopeTick()).unwrap();
                        self.ch4_sender.send(NoiseEvent::EnvelopeTick()).unwrap();
                    }
                    if self.counter == 5 {
                        self.counter = 0;
                    }
                }
                _ => panic!("cant be"),
            }
        }
    }
}

struct Ch1Register {
    //4000
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    //4001
    sweep_change_amount: u8,
    sweep_change_direction: u8,
    sweep_timer_count: u8,
    sweep_enable_flag: u8,

    //4002,4003
    frequency: u16,

    //4003
    key_off_count: u8,
}

impl Ch1Register {
    pub fn new() -> Self {
        Ch1Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            duty: 0,
            sweep_change_amount: 0,
            sweep_change_direction: 0,
            sweep_timer_count: 0,
            sweep_enable_flag: 0,
            frequency: 0,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => {
                self.volume = value & 0x0F; // 0がmin, 15がmax
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
                self.duty = (value & 0xC0) >> 6; // 00:12.5%  01:25%  10:50%  11:75%
            }
            0x4001 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_change_direction = (value & 0x08) >> 3;
                self.sweep_enable_flag = (value & 0x70) >> 4;
                self.sweep_timer_count = (value & 0x80) >> 7;
            }
            0x4002 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4003 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("cant be"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]

struct Envelope {
    rate: u8,
    enabled_sound: bool,
    loop_flag: bool,

    counter: u8,
    division_period: u8,
}

impl Envelope {
    fn new(rate: u8, enabled_sound: bool, loop_flag: bool) -> Self {
        Envelope {
            rate,
            enabled_sound,
            loop_flag,
            counter: 0x0F,
            division_period: rate + 1,
        }
    }

    fn tick(&mut self) {
        self.division_period -= 1;

        if self.division_period != 0 {
            return;
        }

        //division_period = 0は分周期が励起されたときということ
        //分周期は一周期ごとに0に励起される
        if self.counter != 0 {
            self.counter -= 1;
        } else if self.counter == 0 {
            if self.loop_flag {
                self.counter = 0x0F;
            }
        }
        self.division_period = self.rate + 1;
    }

    fn volume(&self) -> f32 {
        (if self.enabled_sound {
            self.counter
        } else {
            self.rate
        }) as f32
            / 15.0
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SquareEvent {
    Note(SquareNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
}

#[derive(Debug, Clone, PartialEq)]
struct SquareNote {
    hz: f32,
    duty: f32, //波の上と下の比率
}
struct SquareWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<SquareEvent>,
    enabled_sound: bool,
    note: SquareNote,
    envelope: Envelope,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //矩形波の生成
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(SquareEvent::Note(note)) => self.note = note,
                    Ok(SquareEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(SquareEvent::Envelope(envelope)) => self.envelope = envelope,
                    Ok(SquareEvent::EnvelopeTick()) => self.envelope.tick(),
                    Err(_) => break,
                }
            }

            *x = if self.phase <= self.note.duty {
                self.envelope.volume()
            } else {
                -self.envelope.volume()
            };

            if !self.enabled_sound {
                *x = 0.0;
            }

            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<SquareEvent>();

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
            enabled_sound: true,
            note: SquareNote { hz: 0.0, duty: 0.0 },
            envelope: Envelope::new(0, false, false),
        })
        .unwrap();

    device.resume();
    (device, sender)
}

struct Ch2Register {
    //4004
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    //4005
    sweep_change_amount: u8,
    sweep_change_direction: u8,
    sweep_timer_count: u8,
    sweep_enable_flag: u8,

    //4006,4007
    frequency: u16,

    //4007
    key_off_count: u8,
}

impl Ch2Register {
    pub fn new() -> Self {
        Ch2Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            duty: 0,
            sweep_change_amount: 0,
            sweep_change_direction: 0,
            sweep_timer_count: 0,
            sweep_enable_flag: 0,
            frequency: 0,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4004 => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
                self.duty = (value & 0xC0) >> 6;
            }
            0x4005 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_change_direction = (value & 0x08) >> 3;
                self.sweep_enable_flag = (value & 0x70) >> 4;
                self.sweep_timer_count = (value & 0x80) >> 7;
            }
            0x4006 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4007 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("cant be"),
        }
    }
}

struct Ch3Register {
    //4008
    length: u8,
    key_off_counter_flag: bool,

    //400A,400B
    frequency: u16,

    //400B
    key_off_count: u8,
}

impl Ch3Register {
    pub fn new() -> Self {
        Ch3Register {
            length: 0,
            key_off_counter_flag: false,
            frequency: 0,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4008 => {
                self.length = value & 0x7F;
                self.key_off_counter_flag = (value & 0x80) == 0;
            }
            0x4009 => {}
            0x400A => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x400B => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("cant be"),
        }
    }
}

enum TriangleEvent {
    Note(TriangleNote),
    Enable(bool),
}

pub struct TriangleNote {
    hz: f32,
}
struct TriangleWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<TriangleEvent>,

    enabled_sound: bool,
    note: TriangleNote,
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //三角波の生成
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(TriangleEvent::Note(note)) => self.note = note,
                    Ok(TriangleEvent::Enable(b)) => self.enabled_sound = b,
                    Err(_) => break,
                }
            }

            *x = (if self.phase <= 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } - 0.25)
                * 4.0;

            if !self.enabled_sound {
                *x = 0.0;
            }

            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

fn init_triangle(sdl_context: &sdl2::Sdl) -> (AudioDevice<TriangleWave>, Sender<TriangleEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<TriangleEvent>();

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
            enabled_sound: true,
            note: TriangleNote { hz: 0.0 },
        })
        .unwrap();

    device.resume();
    (device, sender)
}

enum NoiseKind {
    Long,
    Short,
}

lazy_static! {
    pub static ref NOISE_TABLE: Vec<u16> = vec![
        0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0030, 0x0040, 0x0050, 0x0065, 0x007f, 0x00be,
        0x00fe, 0x017d, 0x01fc, 0x03f9, 0x07f2,
    ];
}

struct Ch4Register {
    //400C
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,

    //400E
    noise_hz: u8,
    kind: NoiseKind,

    //400F
    key_off_count: u8,
}

impl Ch4Register {
    pub fn new() -> Self {
        Ch4Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            noise_hz: 0,
            kind: NoiseKind::Long,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x400C => {
                // 下位4ビット
                self.volume = value & 0x0F;

                // 上位1ビット
                self.envelope_flag = (value & 0x10) == 0;

                // 上位2ビット
                self.key_off_counter_flag = (value & 0x20) == 0;
            }
            0x400E => {
                self.noise_hz = value & 0x0F;
                self.kind = match value & 0x80 {
                    0 => NoiseKind::Long,
                    _ => NoiseKind::Short,
                };
            }
            0x400F => {
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("cant be"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum NoiseEvent {
    Note(NoiseNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseNote {
    hz: f32,
    is_long: bool,
    volume: f32,
}
struct NoiseWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<NoiseEvent>,
    is_sound: bool,
    long_random: NoiseRandom,
    short_random: NoiseRandom,

    enabled_sound: bool,
    envelope: Envelope,
    note: NoiseNote,
}

impl AudioCallback for NoiseWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //ノイズの生成
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(NoiseEvent::Note(note)) => self.note = note,
                    Ok(NoiseEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(NoiseEvent::Envelope(e)) => self.envelope = e,
                    Ok(NoiseEvent::EnvelopeTick()) => self.envelope.tick(),
                    Err(_) => break,
                }
            }
            *x = if self.is_sound { 0.0 } else { 1.0 } * self.envelope.volume();

            if !self.enabled_sound {
                *x = 0.0;
            }

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

fn init_noise(sdl_context: &sdl2::Sdl) -> (AudioDevice<NoiseWave>, Sender<NoiseEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<NoiseEvent>();

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
            long_random: NoiseRandom::new_long(),
            short_random: NoiseRandom::new_short(),
            enabled_sound: true,
            envelope: Envelope::new(0, false, false),
            note: NoiseNote {
                hz: 0.0,
                volume: 0.0,
                is_long: true,
            },
        })
        .unwrap();

    device.resume();
    (device, sender)
}

bitflags! {
    pub struct FrameCounter: u8 {
        const DISABLE_IRQ       = 0b0100_0000;
        const SEQUENCER_MODE    = 0b1000_0000;
    }
}

impl FrameCounter {
    pub fn new() -> Self {
        FrameCounter::from_bits_truncate(0b1100_0000)
    }

    pub fn mode(&self) -> u8 {
        if self.contains(FrameCounter::SEQUENCER_MODE) {
            5
        } else {
            4
        }
    }

    pub fn irq(&self) -> bool {
        !self.contains(FrameCounter::DISABLE_IRQ)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        const ENABLE_1CH        = 0b0000_0001;
        const ENABLE_2CH        = 0b0000_0010;
        const ENABLE_3CH        = 0b0000_0100;
        const ENABLE_4CH        = 0b0000_1000;
        const ENABLE_5CH        = 0b0001_0000;
        const ENABLE_FRAME_IRQ  = 0b0100_0000;
        const ENABLE_DMC_IRQ    = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}
