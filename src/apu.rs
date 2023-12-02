use bitflags::bitflags;
use log::info;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

mod dmc;
use self::dmc::{init_DMC, DMCEvent, DMCWave};
use dmc::DMCRegister;

pub struct NesAPU {
    ch1_register: Ch1Register,
    ch2_register: Ch2Register,
    ch3_register: Ch3Register,
    ch4_register: Ch4Register,
    dmc_register: DMCRegister,

    frame_counter: FrameCounter,
    status: StatusRegister,
    cycles: usize,
    counter: usize,

    ch1_device: AudioDevice<SquareWave>,
    ch1_sender: Sender<SquareEvent>,
    ch1_receiver: Receiver<ChannelEvent>,
    ch1_length_counter: u32,

    ch2_device: AudioDevice<SquareWave>,
    ch2_sender: Sender<SquareEvent>,
    ch2_receiver: Receiver<ChannelEvent>,
    ch2_length_counter: u32,

    ch3_device: AudioDevice<TriangleWave>,
    ch3_sender: Sender<TriangleEvent>,
    ch3_receiver: Receiver<ChannelEvent>,
    ch3_length_counter: u32,

    ch4_device: AudioDevice<NoiseWave>,
    ch4_sender: Sender<NoiseEvent>,
    ch4_receiver: Receiver<ChannelEvent>,
    ch4_length_counter: u32,

    dmc_device: AudioDevice<DMCWave>,
    dmc_sender: Sender<DMCEvent>,
    dmc_receiver: Receiver<ChannelEvent>,
    dmc_sample_byte_count: u32,
}

const NES_CPU_CLOCK: f32 = 1_789_772.5; //1.78MHz

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender, ch1_receiver) = init_square(&sdl_context);
        let (ch2_device, ch2_sender, ch2_receiver) = init_square(&sdl_context);
        let (ch3_device, ch3_sender, ch3_receiver) = init_triangle(&sdl_context);
        let (ch4_device, ch4_sender, ch4_receiver) = init_noise(&sdl_context);
        let (dmc_device, dmc_sender, dmc_receiver) = init_DMC(&sdl_context);
        NesAPU {
            ch1_register: Ch1Register::new(),
            ch2_register: Ch2Register::new(),
            ch3_register: Ch3Register::new(),
            ch4_register: Ch4Register::new(),
            dmc_register: DMCRegister::new(),

            frame_counter: FrameCounter::new(),
            status: StatusRegister::new(),
            cycles: 0,
            counter: 0,

            ch1_device: ch1_device,
            ch1_sender: ch1_sender,
            ch1_receiver: ch1_receiver,
            ch1_length_counter: 0,

            ch2_device: ch2_device,
            ch2_sender: ch2_sender,
            ch2_receiver: ch2_receiver,
            ch2_length_counter: 0,

            ch3_device: ch3_device,
            ch3_sender: ch3_sender,
            ch3_receiver: ch3_receiver,
            ch3_length_counter: 0,

            ch4_device: ch4_device,
            ch4_sender: ch4_sender,
            ch4_receiver: ch4_receiver,
            ch4_length_counter: 0,

            dmc_device: dmc_device,
            dmc_sender: dmc_sender,
            dmc_receiver: dmc_receiver,
            dmc_sample_byte_count: 0,
        }
    }

    pub fn write_1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);

        //sdlに送る

        if addr == 0x4000 {
            self.ch1_sender
                .send(SquareEvent::Note(SquareNote {
                    duty: self.ch1_register.duty,
                }))
                .unwrap();
            self.ch1_sender
                .send(SquareEvent::Envelope(EnvelopeData::new(
                    self.ch1_register.volume,
                    self.ch1_register.envelope_flag,
                    !self.ch1_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x4000 || addr == 0x4003 {
            self.ch1_sender
                .send(SquareEvent::LengthCounter(LengthCounterData::new(
                    self.ch1_register.key_off_count,
                    self.ch1_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x4001 {
            self.ch1_sender
                .send(SquareEvent::Sweep(SweepData::new(
                    self.ch1_register.sweep_change_amount,
                    self.ch1_register.sweep_change_direction,
                    self.ch1_register.sweep_timer_count,
                    self.ch1_register.sweep_enable_flag,
                )))
                .unwrap();
        }

        // 4002か4003のとき、frequencyを更新
        if addr == 0x4002 || addr == 0x4003 {
            self.ch1_sender
                .send(SquareEvent::ChangeFrequency(self.ch1_register.frequency))
                .unwrap();
        }

        //最後のレジスタに書かれているときはリセット
        if addr == 0x4003 {
            self.ch1_sender.send(SquareEvent::Reset()).unwrap();
        }
    }

    pub fn write_2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);

        //sdlに送る

        if addr == 0x4004 {
            self.ch2_sender
                .send(SquareEvent::Note(SquareNote {
                    duty: self.ch2_register.duty,
                }))
                .unwrap();

            self.ch2_sender
                .send(SquareEvent::Envelope(EnvelopeData::new(
                    self.ch2_register.volume,
                    self.ch2_register.envelope_flag,
                    !self.ch2_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x4004 || addr == 0x4007 {
            self.ch2_sender
                .send(SquareEvent::LengthCounter(LengthCounterData::new(
                    self.ch2_register.key_off_count,
                    self.ch2_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x4005 {
            self.ch2_sender
                .send(SquareEvent::Sweep(SweepData::new(
                    self.ch2_register.sweep_change_amount,
                    self.ch2_register.sweep_change_direction,
                    self.ch2_register.sweep_timer_count,
                    self.ch2_register.sweep_enable_flag,
                )))
                .unwrap();
        }

        // 4006か4007のとき、frequencyを更新
        if addr == 0x4006 || addr == 0x4007 {
            self.ch2_sender
                .send(SquareEvent::ChangeFrequency(self.ch2_register.frequency))
                .unwrap();
        }

        //最後のレジスタに書かれているときはリセット
        if addr == 0x4007 {
            self.ch2_sender.send(SquareEvent::Reset()).unwrap();
        }
    }

    pub fn write_3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);

        //sdlに送る

        if addr == 0x4008 {
            self.ch3_sender
                .send(TriangleEvent::LinearCounter(LinearCounterData::new(
                    self.ch3_register.length,
                    self.ch3_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x4008 || addr == 0x400B {
            self.ch3_sender
                .send(TriangleEvent::LengthCounter(LengthCounterData::new(
                    self.ch3_register.key_off_count,
                    self.ch3_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x400A || addr == 0x400B {
            self.ch3_sender
                .send(TriangleEvent::Note(TriangleNote {
                    frequency: self.ch3_register.frequency,
                }))
                .unwrap();
        }

        //最後のレジスタに書かれているときはリセット
        if addr == 0x400B {
            self.ch3_sender.send(TriangleEvent::Reset()).unwrap();
        }
    }

    pub fn write_4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);

        //sdlに送る

        if addr == 0x400C {
            self.ch4_sender
                .send(NoiseEvent::Envelope(EnvelopeData::new(
                    self.ch4_register.volume,
                    self.ch4_register.envelope_flag,
                    !self.ch4_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x400C || addr == 0x400F {
            self.ch4_sender
                .send(NoiseEvent::LengthCounter(LengthCounterData::new(
                    self.ch4_register.key_off_count,
                    self.ch4_register.key_off_counter_flag,
                )))
                .unwrap();
        }

        if addr == 0x400E {
            self.ch4_sender
                .send(NoiseEvent::Note(NoiseNote {
                    hz: self.ch4_register.noise_hz,
                    kind: self.ch4_register.kind,
                }))
                .unwrap();
        }

        //最後のレジスタに書かれているときはリセット
        if addr == 0x400F {
            self.ch4_sender.send(NoiseEvent::Reset()).unwrap();
        }
    }

    pub fn write_dmc(&mut self, addr: u16, value: u8) {
        info!("write dmc addr {:04X} value {:02X}", addr, value);
        self.dmc_register.write(addr, value);

        if addr == 0x4010 {
            self.dmc_sender
                .send(DMCEvent::IrqEnable(self.dmc_register.irq_enable))
                .unwrap();
            self.dmc_sender
                .send(DMCEvent::Loop(self.dmc_register.loop_flag))
                .unwrap();
            self.dmc_sender
                .send(DMCEvent::Frequency(self.dmc_register.frequency_index))
                .unwrap();
        }

        if addr == 0x4011 {
            self.dmc_sender
                .send(DMCEvent::Delta(self.dmc_register.delta_counter))
                .unwrap();
        }

        if addr == 0x4012 {
            self.dmc_sender
                .send(DMCEvent::SampleStartAddr(
                    self.dmc_register.sample_start_addr,
                ))
                .unwrap();
        }

        if addr == 0x4013 {
            self.dmc_sender
                .send(DMCEvent::SampleByteCount(
                    self.dmc_register.sample_byte_count,
                ))
                .unwrap();
        }

        //最後のレジスタに書かれているときはリセット
        if addr == 0x4013 {
            self.dmc_sender.send(DMCEvent::Reset()).unwrap();
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut res = self.status.bits();
        self.receive_events();

        // lengthcounterの値を見てフラグを立てるか立てないかの処理をする
        // 下の4bitを一旦0に落とす
        res = res & 0xF0;
        res = res | if self.ch1_length_counter == 0 { 0 } else { 1 };
        res = res | (if self.ch2_length_counter == 0 { 0 } else { 1 } << 1);
        res = res | (if self.ch3_length_counter == 0 { 0 } else { 1 } << 2);
        res = res | (if self.ch4_length_counter == 0 { 0 } else { 1 } << 3);
        res = res
            | (if self.dmc_sample_byte_count == 0 {
                0
            } else {
                1
            } << 4);

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

        self.dmc_sender
            .send(DMCEvent::Enable(
                self.status.contains(StatusRegister::ENABLE_DMC),
            ))
            .unwrap();
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter.update(value);

        //4017への書き込みによって分周期とシーケンサをリセットする
        self.cycles = 0;
        self.counter = 0;
    }

    fn receive_events(&mut self) {
        loop {
            let res = self.ch1_receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(ChannelEvent::LengthCounter(counter)) => {
                    self.ch1_length_counter = counter;
                }
                _ => break,
            }
        }

        loop {
            let res = self.ch2_receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(ChannelEvent::LengthCounter(counter)) => {
                    self.ch2_length_counter = counter;
                }
                _ => break,
            }
        }

        loop {
            let res = self.ch3_receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(ChannelEvent::LengthCounter(counter)) => {
                    self.ch3_length_counter = counter;
                }
                _ => break,
            }
        }

        loop {
            let res = self.ch4_receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(ChannelEvent::LengthCounter(counter)) => {
                    self.ch4_length_counter = counter;
                }
                _ => break,
            }
        }

        loop {
            let res = self.dmc_receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(ChannelEvent::LengthCounter(counter)) => {
                    self.dmc_sample_byte_count = counter;
                }
                _ => break,
            }
        }
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

            self.receive_events();

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

        //三角波の線形カウンタのクロック生成
        self.ch3_sender
            .send(TriangleEvent::LinearCounterTick())
            .unwrap();
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
    Envelope(EnvelopeData),
    EnvelopeTick(),
    LengthCounter(LengthCounterData),
    LengthCounterTick(),
    ChangeFrequency(u16),
    Sweep(SweepData),
    SweepTick(),
    Reset(),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelEvent {
    LengthCounter(u32),
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
    sender: Sender<ChannelEvent>,
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
                    Ok(SquareEvent::Envelope(envelope)) => self.envelope.data = envelope,
                    Ok(SquareEvent::EnvelopeTick()) => self.envelope.tick(),
                    Ok(SquareEvent::LengthCounter(l)) => {
                        // info!(
                        //     "OW Length Counter {} {}",
                        //     self.length_counter.enabled, self.length_counter.count
                        // );
                        self.length_counter.data = l
                    }
                    Ok(SquareEvent::LengthCounterTick()) => {
                        // info!(
                        //     "Length Counter {} {}",
                        //     self.length_counter.enabled, self.length_counter.counter
                        // );
                        self.length_counter.tick();
                        self.sender
                            .send(ChannelEvent::LengthCounter(
                                self.length_counter.counter as u32,
                            ))
                            .unwrap();
                    }
                    Ok(SquareEvent::ChangeFrequency(f)) => {
                        self.sweep.frequency = f;
                    }
                    Ok(SquareEvent::Sweep(s)) => self.sweep.data = s,
                    Ok(SquareEvent::SweepTick()) => self.sweep.tick(&mut self.length_counter),
                    Ok(SquareEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                        self.sweep.reset();
                        self.phase = 0.0;
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
fn init_square(
    sdl_context: &sdl2::Sdl,
) -> (
    AudioDevice<SquareWave>,
    Sender<SquareEvent>,
    Receiver<ChannelEvent>,
) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<SquareEvent>();
    let (sender2, receiver2) = channel::<ChannelEvent>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None, //0サンプルごとにcallbackが呼ばれる(1秒間に44100回callbackが呼ばれる)
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| SquareWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            enabled_sound: true,
            note: SquareNote::new(),
            envelope: Envelope::new(),
            length_counter: LengthCounter::new(),
            sweep: Sweep::new(),
        })
        .unwrap();

    device.resume();
    (device, sender, receiver2)
}

enum TriangleEvent {
    Note(TriangleNote),
    Enable(bool),
    LengthCounter(LengthCounterData),
    LengthCounterTick(),
    LinearCounter(LinearCounterData),
    LinearCounterTick(),
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
    sender: Sender<ChannelEvent>,

    enabled_sound: bool,
    note: TriangleNote,

    length_counter: LengthCounter,
    linear_counter: LinearCounter, // 音長コントロール
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    // outは外からもらってくるオーディオのバッファ.ここに波を入れる
    fn callback(&mut self, out: &mut [Self::Channel]) {
        //三角波の生成3
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(TriangleEvent::Note(note)) => self.note = note,
                    Ok(TriangleEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(TriangleEvent::LengthCounter(l)) => self.length_counter.data = l,
                    Ok(TriangleEvent::LengthCounterTick()) => {
                        self.length_counter.tick();
                        self.linear_counter.tick();
                        self.sender
                            .send(ChannelEvent::LengthCounter(
                                self.length_counter.counter as u32,
                            ))
                            .unwrap();
                    }
                    Ok(TriangleEvent::LinearCounter(l)) => {
                        self.linear_counter.data = l;
                        self.linear_counter.reset();
                    }
                    Ok(TriangleEvent::LinearCounterTick()) => {
                        self.linear_counter.tick();
                    }
                    Ok(TriangleEvent::Reset()) => {
                        self.length_counter.reset();
                        self.linear_counter.reset();
                        self.phase = 0.0;
                    }
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

            if self.linear_counter.mute() {
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
fn init_triangle(
    sdl_context: &sdl2::Sdl,
) -> (
    AudioDevice<TriangleWave>,
    Sender<TriangleEvent>,
    Receiver<ChannelEvent>,
) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<TriangleEvent>();
    let (sender2, receiver2) = channel::<ChannelEvent>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None, //0サンプルごとにcallbackが呼ばれる(1秒間に44100回callbackが呼ばれる)
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| TriangleWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            enabled_sound: true,
            note: TriangleNote::new(),
            length_counter: LengthCounter::new(),
            linear_counter: LinearCounter::new(),
        })
        .unwrap();

    device.resume();
    (device, sender, receiver2)
}

#[derive(Debug, Clone, PartialEq)]
enum NoiseEvent {
    Note(NoiseNote),
    Enable(bool),
    Envelope(EnvelopeData),
    EnvelopeTick(),
    LengthCounter(LengthCounterData),
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
    sender: Sender<ChannelEvent>,
    random: NoiseRandom,

    enabled_sound: bool,
    envelope: Envelope,
    note: NoiseNote,

    length_counter: LengthCounter,
    is_sound: bool,
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
                    Ok(NoiseEvent::Note(note)) => {
                        self.note = note;
                    }
                    Ok(NoiseEvent::Enable(b)) => self.enabled_sound = b,
                    Ok(NoiseEvent::Envelope(e)) => self.envelope.data = e,
                    Ok(NoiseEvent::EnvelopeTick()) => self.envelope.tick(),
                    Ok(NoiseEvent::LengthCounter(l)) => self.length_counter.data = l,
                    Ok(NoiseEvent::LengthCounterTick()) => {
                        self.length_counter.tick();
                        self.sender
                            .send(ChannelEvent::LengthCounter(
                                self.length_counter.counter as u32,
                            ))
                            .unwrap();
                    }
                    Ok(NoiseEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                        self.phase = 0.0;
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
            let mut add = self.note.hz() / self.freq;
            self.phase = (self.phase + add) % 1.0;

            loop {
                if add < 1.0 {
                    break;
                }
                add -= 1.0;
                self.is_sound = self.random.next(self.note.is_long())
            }

            if last_phase > self.phase {
                self.is_sound = self.random.next(self.note.is_long())
            }
            self.phase = (self.phase + 1.0) % self.freq; //44100hzで1フェーズ
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NoiseKind {
    Long,
    Short,
}

static NOISE_TABLE: [u16; 16] = [
    0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0060, 0x0080, 0x00A0, 0x00CA, 0x00FE, 0x017C, 0x01FC,
    0x02FA, 0x03F8, 0x07F2, 0x0FE4,
];

struct NoiseRandom {
    value: u16,
}

impl NoiseRandom {
    pub fn new() -> Self {
        NoiseRandom { value: 1 }
    }

    pub fn next(&mut self, is_long: bool) -> bool {
        //ロングモード時はビット0とビット1のXORを入れる
        let bit = if is_long { 1 } else { 6 }; //長周期は1,短周期は6
        let b = (self.value & 0x01) ^ ((self.value >> bit) & 0x01);
        self.value = self.value >> 1;
        self.value = self.value & 0b011_1111_1111_1111 | b << 14;

        //シフトレジスタのビット0が1なら出力は0
        self.value & 0x01 != 0
    }
}

//sdl周りの初期化
fn init_noise(
    sdl_context: &sdl2::Sdl,
) -> (
    AudioDevice<NoiseWave>,
    Sender<NoiseEvent>,
    Receiver<ChannelEvent>,
) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<NoiseEvent>();
    let (sender2, receiver2) = channel::<ChannelEvent>();

    let desire_spec = AudioSpecDesired {
        freq: Some(44100), //1秒間に44100個の配列が消費される
        channels: Some(1),
        samples: None, //0サンプルごとにcallbackが呼ばれる(1秒間に44100回callbackが呼ばれる)
    };

    let device = audio_subsystem
        .open_playback(None, &desire_spec, |spec| NoiseWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            random: NoiseRandom::new(),
            enabled_sound: true,
            envelope: Envelope::new(),
            note: NoiseNote::new(),
            length_counter: LengthCounter::new(),
            is_sound: false,
        })
        .unwrap();

    device.resume();
    (device, sender, receiver2)
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
        const ENABLE_DMC        = 0b0001_0000;
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
struct EnvelopeData {
    // registerの値を持ってきているもの
    rate: u8,
    enabled_sound: bool,
    loop_flag: bool,
}

impl EnvelopeData {
    fn new(rate: u8, enabled_sound: bool, loop_flag: bool) -> Self {
        EnvelopeData {
            rate,
            enabled_sound,
            loop_flag,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Envelope {
    data: EnvelopeData,

    // 随時変わるもの
    counter: u8,
    division_period: u8,
}

impl Envelope {
    fn new() -> Self {
        Envelope {
            data: EnvelopeData::new(0, false, false),
            counter: 0x0F,
            division_period: 1,
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
            if self.data.loop_flag {
                self.reset();
            }
        }
        self.division_period = self.data.rate + 1;
    }

    fn volume(&self) -> f32 {
        (if self.data.enabled_sound {
            self.counter
        } else {
            self.data.rate
        }) as f32
            / 15.0
    }

    fn reset(&mut self) {
        self.counter = 0x0F;
        self.division_period = self.data.rate + 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
// 音の鳴る時間
struct LengthCounterData {
    enabled: bool,
    counter_for_reset: u8, //最初のカウント値。リセット用
}
impl LengthCounterData {
    fn new(counter_for_reset: u8, enabled: bool) -> Self {
        LengthCounterData {
            enabled,
            counter_for_reset: LENGTH_COUNTER_TABLE[counter_for_reset as usize] + 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
// 音の鳴る時間
struct LengthCounter {
    counter: u8,
    data: LengthCounterData,
}

impl LengthCounter {
    fn new() -> Self {
        LengthCounter {
            counter: 0,
            data: LengthCounterData::new(0, false),
        }
    }

    fn tick(&mut self) {
        if !self.data.enabled {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn mute(&self) -> bool {
        self.counter == 0
    }

    fn reset(&mut self) {
        self.counter = self.data.counter_for_reset;
    }
}

static LENGTH_COUNTER_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

#[derive(Debug, Clone, PartialEq)]
struct SweepData {
    change_amount: u8,
    change_direction: u8,
    timer_count: u8,
    enabled: bool,
}

impl SweepData {
    fn new(change_amount: u8, change_direction: u8, timer_count: u8, enabled: bool) -> Self {
        SweepData {
            change_amount,
            change_direction,
            timer_count,
            enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Sweep {
    data: SweepData,
    frequency: u16,
    counter: u8,
}

impl Sweep {
    fn new() -> Self {
        Sweep {
            data: SweepData::new(0, 0, 0, false),
            frequency: 0,
            counter: 0,
        }
    }

    fn tick(&mut self, length_counter: &mut LengthCounter) {
        self.counter += 1;
        if self.counter < (self.data.timer_count + 1) {
            return;
        }
        self.counter = 0;

        if !self.data.enabled {
            return;
        }

        if self.data.change_amount == 0 {
            return;
        }

        //length_counter が0ではない
        if length_counter.mute() {
            return;
        }

        if self.data.change_direction == 0 {
            //尻下がりモード
            self.frequency = self.frequency + (self.frequency >> self.data.change_amount as u16);
        } else {
            //尻上がりモード
            self.frequency = self.frequency - (self.frequency >> self.data.change_amount as u16);
        }

        //チャンネルの周期が8未満か0x7FFより大きくなったらスイープを停止しチャンネルを無音化する
        if self.frequency < 0x008 || self.frequency > 0x07FF {
            length_counter.counter = 0;
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0;
        }
        NES_CPU_CLOCK / (16.0 * (self.frequency as f32 + 1.0))
    }

    fn reset(&mut self) {
        // self.frequency = self.data.org_freq;
        self.counter = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
//LengthCounterのテーブルを見に行かないもの
struct LinearCounterData {
    counter_for_reset: u8, //最初のカウント値。リセット用
    enabled: bool,
}

impl LinearCounterData {
    fn new(counter_for_reset: u8, enabled: bool) -> Self {
        LinearCounterData {
            counter_for_reset,
            enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
//LengthCounterのテーブルを見に行かないもの
struct LinearCounter {
    data: LinearCounterData,
    counter: u8,
}

impl LinearCounter {
    fn new() -> Self {
        LinearCounter {
            data: LinearCounterData::new(0, false),
            counter: 0,
        }
    }

    fn tick(&mut self) {
        if !self.data.enabled {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn mute(&self) -> bool {
        self.counter == 0
    }

    fn reset(&mut self) {
        self.counter = self.data.counter_for_reset;
    }
}
