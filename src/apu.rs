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

        //sdlに送る
        self.ch1_sender
            .send(SquareEvent::Note(SquareNote {
                duty: self.ch1_register.duty,
            }))
            .unwrap();

        self.ch1_sender
            .send(SquareEvent::Envelope(Envelope::new(
                self.ch1_register.volume,
                self.ch1_register.envelope_flag,
                !self.ch1_register.key_off_counter_flag,
            )))
            .unwrap();

        self.ch1_sender
            .send(SquareEvent::LengthCounter(LengthCounter::new(
                self.ch1_register.key_off_counter_flag,
                self.ch1_register.key_off_count,
            )))
            .unwrap();

        self.ch1_sender
            .send(SquareEvent::Sweep(Sweep::new(
                self.ch1_register.frequency,
                self.ch1_register.sweep_change_amount,
                self.ch1_register.sweep_change_direction,
                self.ch1_register.sweep_timer_count,
                self.ch1_register.sweep_enable_flag,
            )))
            .unwrap();

        //最後のレジスタに書かれているときはリセット
        if addr == 0x4003 {
            self.ch1_sender.send(SquareEvent::Reset()).unwrap();
        }
    }

    pub fn write_2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);

        //sdlに送る
        self.ch2_sender
            .send(SquareEvent::Note(SquareNote {
                duty: self.ch2_register.duty,
            }))
            .unwrap();

        self.ch2_sender
            .send(SquareEvent::Envelope(Envelope::new(
                self.ch2_register.volume,
                self.ch2_register.envelope_flag,
                !self.ch2_register.key_off_counter_flag,
            )))
            .unwrap();

        self.ch2_sender
            .send(SquareEvent::LengthCounter(LengthCounter::new(
                self.ch2_register.key_off_counter_flag,
                self.ch2_register.key_off_count,
            )))
            .unwrap();

        self.ch2_sender
            .send(SquareEvent::Sweep(Sweep::new(
                self.ch2_register.frequency,
                self.ch2_register.sweep_change_amount,
                self.ch2_register.sweep_change_direction,
                self.ch2_register.sweep_timer_count,
                self.ch2_register.sweep_enable_flag,
            )))
            .unwrap();

        //最後のレジスタに書かれているときはリセット
        if addr == 0x4007 {
            self.ch2_sender.send(SquareEvent::Reset()).unwrap();
        }
    }

    pub fn write_3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);

        //sdlに送る
        self.ch3_sender
            .send(TriangleEvent::Note(TriangleNote {
                frequency: self.ch3_register.frequency,
            }))
            .unwrap();

        self.ch3_sender
            .send(TriangleEvent::LengthCounter(LengthCounter::new(
                self.ch3_register.key_off_counter_flag,
                self.ch3_register.key_off_count,
            )))
            .unwrap();

        //最後のレジスタに書かれているときはリセット
        if addr == 0x400B {
            self.ch3_sender.send(TriangleEvent::Reset()).unwrap();
        }
    }

    pub fn write_4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);

        //sdlに送る
        self.ch4_sender
            .send(NoiseEvent::Note(NoiseNote {
                hz: self.ch4_register.noise_hz,
                kind: self.ch4_register.kind,
            }))
            .unwrap();

        self.ch4_sender
            .send(NoiseEvent::Envelope(Envelope::new(
                self.ch4_register.volume,
                self.ch4_register.envelope_flag,
                !self.ch4_register.key_off_counter_flag,
            )))
            .unwrap();

        self.ch4_sender
            .send(NoiseEvent::LengthCounter(LengthCounter::new(
                self.ch4_register.key_off_counter_flag,
                self.ch4_register.key_off_count,
            )))
            .unwrap();

        //最後のレジスタに書かれているときはリセット
        if addr == 0x400F {
            self.ch4_sender.send(NoiseEvent::Reset()).unwrap();
        }
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

    pub fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter.update(value);

        //4017への書き込みによって分周期とシーケンサをリセットする
        self.cycles = 0;
        self.counter = 0;
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
                    // - - - f   60Hz
                    // - l - l  120Hz
                    // e e e e  240Hz
                    if self.counter == 2 || self.counter == 4 {
                        //長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter == 4 {
                        //割り込みフラグセット
                        self.counter = 0;
                        if self.frame_counter.irq() {
                            self.status.insert(StatusRegister::ENABLE_FRAME_IRQ);
                        }
                    }
                    //エンベロープと三角波の線形カウンタのクロックサイクル
                    self.send_envelope_tick();
                }
                5 => {
                    // - - - - -   (割り込みフラグはセットしない)
                    // l - l - -   96Hz
                    // e e e e -  192Hz
                    if self.counter == 1 || self.counter == 3 {
                        // 長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter <= 4 {
                        // エンベロープと三角波の線形カウンタのクロック生成
                        self.send_envelope_tick();
                    }
                    if self.counter == 5 {
                        self.counter = 0;
                    }
                }
                _ => panic!("cant be"),
            }
        }
    }

    //実際にtickを送る処理　~_tick

    fn send_length_counter_tick(&self) {
        self.ch1_sender
            .send(SquareEvent::LengthCounterTick())
            .unwrap();
        self.ch2_sender
            .send(SquareEvent::LengthCounterTick())
            .unwrap();
        self.ch3_sender
            .send(TriangleEvent::LengthCounterTick())
            .unwrap();
        self.ch4_sender
            .send(NoiseEvent::LengthCounterTick())
            .unwrap();
    }

    fn send_envelope_tick(&self) {
        self.ch1_sender.send(SquareEvent::EnvelopeTick()).unwrap();
        self.ch2_sender.send(SquareEvent::EnvelopeTick()).unwrap();
        self.ch4_sender.send(NoiseEvent::EnvelopeTick()).unwrap();
    }

    fn send_sweep_tick(&self) {
        self.ch1_sender.send(SquareEvent::SweepTick()).unwrap();
        self.ch2_sender.send(SquareEvent::SweepTick()).unwrap();
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
    sweep_enable_flag: bool,

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
            sweep_enable_flag: false,
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
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enable_flag = (value & 0x80) != 0;
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
    sweep_enable_flag: bool,

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
            sweep_enable_flag: false,
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
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enable_flag = (value & 0x80) != 0;
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
enum SquareEvent {
    Note(SquareNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Sweep(Sweep),
    SweepTick(),
    Reset(),
}

#[derive(Debug, Clone, PartialEq)]
struct SquareNote {
    duty: u8, //波の上と下の比率
}

impl SquareNote {
    fn new() -> Self {
        SquareNote { duty: 0 }
    }

    fn duty(&self) -> f32 {
        match self.duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.5,
            0x03 => 0.75,
            _ => panic!("cant be"),
        }
    }
}
struct SquareWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<SquareEvent>,
    enabled_sound: bool,
    note: SquareNote,
    envelope: Envelope,
    length_counter: LengthCounter,
    sweep: Sweep,
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
                    Ok(SquareEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(SquareEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(SquareEvent::Sweep(s)) => self.sweep = s,
                    Ok(SquareEvent::SweepTick()) => self.sweep.tick(&self.length_counter),
                    Ok(SquareEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                        self.sweep.reset();
                    }
                    Err(_) => break,
                }
            }

            *x = if self.phase <= self.note.duty() {
                self.envelope.volume()
            } else {
                -self.envelope.volume()
            };

            if self.length_counter.mute() {
                *x = 0.0;
            }

            if !self.enabled_sound {
                *x = 0.0;
            }

            let hz = self.sweep.hz();
            if hz != 0.0 {
                self.phase = (self.phase + self.sweep.hz() / self.freq) % 1.0;
            }
        }
    }
}

//sdl周りの初期化
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
            note: SquareNote::new(),
            envelope: Envelope::new(0, false, false),
            length_counter: LengthCounter::new(false, 0),
            sweep: Sweep::new(0, 0, 0, 0, false),
        })
        .unwrap();

    device.resume();
    (device, sender)
}

enum TriangleEvent {
    Note(TriangleNote),
    Enable(bool),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Reset(),
}

pub struct TriangleNote {
    frequency: u16,
}

impl TriangleNote {
    fn new() -> Self {
        TriangleNote { frequency: 0 }
    }

    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / (32.0 * (self.frequency as f32 + 1.0))
    }
}
struct TriangleWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<TriangleEvent>,

    enabled_sound: bool,
    note: TriangleNote,

    length_counter: LengthCounter,
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
                    Ok(TriangleEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(TriangleEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(TriangleEvent::Reset()) => self.length_counter.reset(),
                    Err(_) => break,
                }
            }

            *x = (if self.phase <= 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } - 0.25)
                * 4.0;

            if self.length_counter.mute() {
                *x = 0.0;
            }

            if !self.enabled_sound {
                *x = 0.0;
            }

            self.phase = (self.phase + self.note.hz() / self.freq) % 1.0;
        }
    }
}

//sdl周りの初期化
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
            note: TriangleNote::new(),
            length_counter: LengthCounter::new(false, 0),
        })
        .unwrap();

    device.resume();
    (device, sender)
}

#[derive(Debug, Clone, PartialEq)]
enum NoiseEvent {
    Note(NoiseNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Reset(),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseNote {
    hz: u8,
    kind: NoiseKind,
}

impl NoiseNote {
    fn new() -> Self {
        NoiseNote {
            hz: 0,
            kind: NoiseKind::Long,
        }
    }

    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / NOISE_TABLE[self.hz as usize] as f32
    }

    fn is_long(&self) -> bool {
        self.kind == NoiseKind::Long
    }
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

    length_counter: LengthCounter,
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
                    Ok(NoiseEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(NoiseEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(NoiseEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                    }
                    Err(_) => break,
                }
            }
            *x = if self.is_sound { 0.0 } else { 1.0 } * self.envelope.volume();

            if self.length_counter.mute() {
                *x = 0.0;
            }

            if !self.enabled_sound {
                *x = 0.0;
            }

            let last_phase = self.phase;
            self.phase = (self.phase + self.note.hz() / self.freq) % 1.0;

            if last_phase > self.phase {
                self.is_sound = if self.note.is_long() {
                    self.long_random.next()
                } else {
                    self.short_random.next()
                };
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

//sdl周りの初期化
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
            note: NoiseNote::new(),
            length_counter: LengthCounter::new(false, 0),
        })
        .unwrap();

    device.resume();
    (device, sender)
}

bitflags! {
    //フレームカウンタ(フレームシーケンサ)
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
                self.reset();
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

    fn reset(&mut self) {
        self.counter = 0x0F;
        self.division_period = self.rate + 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LengthCounter {
    enabled: bool,
    count: u8, //もとのカウント値
    counter: u8,
}

impl LengthCounter {
    fn new(enabled: bool, counter: u8) -> Self {
        LengthCounter {
            enabled,
            counter,
            count: LENGTH_COUNTER_TABLE[counter as usize],
        }
    }

    fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn mute(&self) -> bool {
        self.enabled && self.counter == 0
    }

    fn reset(&mut self) {
        self.counter = self.count;
    }
}

lazy_static! {
    pub static ref LENGTH_COUNTER_TABLE: Vec<u8> = vec![
        0x05, 0x7F, 0x0A, 0x01, 0x14, 0x02, 0x28, 0x03, 0x50, 0x04, 0x1E, 0x05, 0x07, 0x06, 0x0D,
        0x07, 0x06, 0x08, 0x0C, 0x09, 0x18, 0x0A, 0x30, 0x0B, 0x60, 0x0C, 0x24, 0x0D, 0x08, 0x0E,
        0x10, 0x0F,
    ];
}

#[derive(Debug, Clone, PartialEq)]
struct Sweep {
    org_freq: u16,
    frequency: u16,
    change_amount: u8,
    change_direction: u8,
    timer_count: u8,
    enabled: bool,
    counter: u8,
}

impl Sweep {
    fn new(
        frequency: u16,
        change_amount: u8,
        change_direction: u8,
        timer_count: u8,
        enabled: bool,
    ) -> Self {
        Sweep {
            org_freq: frequency,
            frequency,
            change_amount,
            change_direction,
            timer_count,
            enabled,
            counter: 0,
        }
    }

    fn tick(&mut self, length_counter: &LengthCounter) {
        if !self.enabled {
            return;
        }

        if self.change_amount == 0 {
            return;
        }

        //length_counter が0ではない
        if length_counter.mute() {
            return;
        }

        self.counter += 1;
        if self.counter < (self.timer_count + 1) {
            return;
        }
        self.counter = 0;

        if self.change_direction == 0 {
            //尻下がりモード
            self.frequency = self.frequency + (self.frequency >> self.change_amount as u16);
        } else {
            //尻上がりモード
            self.frequency = self.frequency - (self.frequency >> self.change_amount as u16);
        }

        //チャンネルの周期が8未満か0x7FFより大きくなったらスイープを停止しチャンネルを無音化する
        if self.frequency < 8 || self.frequency > 0x7FF {
            self.frequency = 0;
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0;
        }
        NES_CPU_CLOCK / (16.0 * (self.frequency as f32 + 1.0))
    }

    fn reset(&mut self) {
        self.frequency = self.org_freq;
        self.counter = 0;
    }
}
