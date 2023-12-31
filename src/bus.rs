use crate::apu::NesAPU;
use crate::frame::Frame;
use crate::joypad::Joypad;
use crate::ppu::NesPPU;
use crate::MAPPER;
use log::{debug, info};

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    // prg_rom: Vec<u8>,
    ppu: NesPPU,
    frame: Frame,
    joypad1: Joypad,
    // joypad2: Joypad,
    apu: NesAPU,
    cycles: usize,

    game_loop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad, &Frame) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(apu: NesAPU, game_loop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad, &Frame) + 'call,
    {
        let ppu = NesPPU::new();
        Bus {
            cpu_vram: [0; 2048],
            ppu: ppu,
            frame: Frame::new(),
            joypad1: Joypad::new(),
            // joypad2: Joypad::new(),
            apu: apu,
            cycles: 0,
            game_loop_callback: Box::from(game_loop_callback),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3, &mut self.frame);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        self.apu.tick(cycles);

        if !nmi_before && nmi_after {
            (self.game_loop_callback)(&self.ppu, &mut self.joypad1, &self.frame);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<i32> {
        if self.ppu.clear_nmi_interrupt {
            self.ppu.clear_nmi_interrupt = false;
            self.ppu.nmi_interrupt = None;
            return None;
        }
        let res = self.ppu.nmi_interrupt;
        self.ppu.nmi_interrupt = None;
        res
    }

    pub fn poll_apu_irq(&mut self) -> bool {
        self.apu.irq()
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
// const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
}

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                //0x0000 ~ 0x1fff
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }

            0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                0
                // panic!("Attempt to read from write-only PPU addr {:X}", addr);
            }
            0x2000 => self.ppu.read_ctrl(),
            0x2001 => self.ppu.read_mask(),
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                debug!("READ PPU MIRROR: {:04X} => {:04X}", addr, mirror_down_addr);
                self.mem_read(mirror_down_addr)
            }

            0x4015 => self.apu.read_status(),
            0x4016 => self.joypad1.read(),
            0x4017 => 0,

            0x6000..=0x7FFF => unsafe { MAPPER.read_prg_ram(addr) },

            PRG_ROM..=PRG_ROM_END => {
                unsafe { MAPPER.read_prg_rom(addr) }
                // self.read_prg_rom(addr)
            }

            _ => {
                println!("Ignoring mem access at {:X}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }

            0x2000 => self.ppu.write_to_ctrl(data),
            0x2001 => self.ppu.write_to_mask(data),
            0x2002 => self.ppu.write_to_status(data), //read only?
            0x2003 => self.ppu.write_to_oam_addr(data),
            0x2004 => self.ppu.write_to_oam_data(data),
            0x2005 => self.ppu.write_to_scroll(data),
            0x2006 => self.ppu.write_to_ppu_addr(data),
            0x2007 => self.ppu.write_to_data(data),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }

            0x4000..=0x4003 => {
                // APU 1ch
                self.apu.write_1ch(addr, data)
            }

            0x4004..=0x4007 => {
                // APU 2ch
                self.apu.write_2ch(addr, data)
            }

            0x4008 | 0x400A | 0x400B => {
                // APU 3ch
                self.apu.write_3ch(addr, data)
            }

            0x400C | 0x400E | 0x400F => {
                // APU 4ch
                self.apu.write_4ch(addr, data)
            }

            0x4010..=0x4013 => {
                // DMC
                self.apu.write_dmc(addr, data)
            }

            0x4014 => {
                let mut values: [u8; 256] = [0; 256];
                for i in 0x00..=0xFF {
                    values[i] = self.mem_read((data as u16) << 8 | i as u16);
                }
                self.ppu.write_to_oam_dma(values);
                for _ in 0..513 {
                    self.ppu.tick(1, &mut self.frame);
                }
            }

            0x4015 => {
                self.apu.write_status(data);
            }

            0x4016 => {
                self.joypad1.write(data);
            }

            0x4017 => {
                self.apu.write_frame_counter(data);
                info!("WRITE ACCESS 0x4017. {:02X}", data);
            }

            0x6000..=0x7FFF => unsafe { MAPPER.write_prg_ram(addr, data) },

            PRG_ROM..=PRG_ROM_END => unsafe {
                MAPPER.write(addr, data);
            },

            _ => {
                println!("Ignoring mem write-access at {:X}", addr)
            }
        }
    }
}
