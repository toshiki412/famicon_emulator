use crate::rom::{Mirroring, Rom};
use log::{debug, info, trace};
use std::{fs::File, io::Write};

pub fn create_mapper(rom: Rom) -> Box<dyn Mapper> {
    let mut mapper: Box<dyn Mapper> = match rom.mapper {
        0 => Box::new(Mapper0::new()),
        1 => Box::new(Mapper1::new()),
        2 => Box::new(Mapper2::new()),
        3 => Box::new(Mapper3::new()),
        4 => Box::new(Mapper4::new()),
        _ => panic!("Not support mapper"),
    };

    mapper.set_rom(rom);
    return mapper;
}

pub trait Mapper: Send {
    //インターフェースだけを定義
    fn set_rom(&mut self, rom: Rom);
    fn is_chr_ram(&mut self) -> bool;

    fn write(&mut self, addr: u16, data: u8);
    fn mirroring(&self) -> Mirroring;

    fn write_prg_ram(&mut self, addr: u16, data: u8);
    fn read_prg_ram(&self, addr: u16) -> u8;
    fn load_prg_ram(&mut self, raw: &Vec<u8>);

    fn read_prg_rom(&self, addr: u16) -> u8;
    fn write_chr_rom(&mut self, addr: u16, value: u8);
    fn read_chr_rom(&self, addr: u16) -> u8;

    fn scanline(&mut self, scanline: usize, show_background: bool);
    fn is_irq(&mut self) -> bool;
}

pub struct Mapper0 {
    pub rom: Rom,
}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 { rom: Rom::empty() }
    }
}

impl Mapper for Mapper0 {
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom;
    }
    fn write(&mut self, _addr: u16, _data: u8) {
        //何もしない
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn write_prg_ram(&mut self, _addr: u16, _data: u8) {}
    fn read_prg_ram(&self, _addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, _raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut mirror_addr = addr - 0x8000;
        //programのromは16kB刻み prg_rom.len() == 0x4000は16kB
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            //mirror if needed
            mirror_addr = addr % 0x4000;
        }
        self.rom.prg_rom[mirror_addr as usize]
    }

    fn write_chr_rom(&mut self, _addr: u16, _value: u8) {}
    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
    fn scanline(&mut self, _scanline: usize, _show_background: bool) {}
    fn is_irq(&mut self) -> bool {
        false
    }
}

pub struct Mapper1 {
    pub rom: Rom,
    prg_ram: Vec<u8>,

    shift_register: u8,
    shift_count: u8,

    control: u8,   //内部レジスタ 8000~9FFF
    chr_bank0: u8, //内部レジスタ A000~BFFF
    chr_bank1: u8, //内部レジスタ C000~DFFF
    prg_bank: u8,  //内部レジスタ E000~FFFF
}

//SUROM
impl Mapper1 {
    pub fn new() -> Self {
        Mapper1 {
            rom: Rom::empty(),
            prg_ram: vec![0xFF; 8192], //8kiB
            shift_register: 0x10,
            shift_count: 0,
            control: 0x0C,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }

    fn reset(&mut self) {
        self.shift_register = 0x10;
        self.shift_count = 0;
    }

    fn chr_rom_addr(&self, addr: u16) -> usize {
        addr as usize

        // SUROM以外には必要
        // let bank_size = 4 * 1024 as usize; //4kB
        // match (self.control & 0x10) >> 4 {
        //     0 => {
        //         //一度に8kBを切り替える
        //         let bank = self.chr_bank0 & 0x1F;
        //         addr as usize + bank_size * bank as usize
        //     }
        //     1 => match addr {
        //         0x0000..=0x0FFF => {
        //             let bank = self.chr_bank0 & 0x1F;
        //             addr as usize + bank_size * bank as usize
        //         }
        //         0x1000..=0x1FFF => {
        //             let bank = self.chr_bank1 & 0x1F;
        //             (addr as usize - 0x1000) + bank_size * bank as usize
        //         }
        //         _ => panic!("cant be"),
        //     },
        //     _ => panic!("cant be"),
        // }
    }
}
impl Mapper for Mapper1 {
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn set_rom(&mut self, rom: Rom) {
        self.load_prg_ram(&rom.save_data);
        self.rom = rom;
    }
    fn write(&mut self, addr: u16, data: u8) {
        // ex           data      shift_register
        //              000edcba  10000
        // sta $XXXX    000edcba->a1000
        // lsa a        0000edcb  a1000

        if data & 0x80 != 0 {
            self.reset();
            return;
        }

        self.shift_register = self.shift_register >> 1;
        self.shift_register = self.shift_register | ((data & 0x01) << 4);
        self.shift_count += 1;

        //5回目の書き込みだけが重要で1~4回目は8000~FFFFの間のアドレスであればどこに書き込んでも良い
        if self.shift_count == 5 {
            match addr {
                0x8000..=0x9FFF => self.control = self.shift_register,
                0xA000..=0xBFFF => self.chr_bank0 = self.shift_register,
                0xC000..=0xDFFF => self.chr_bank1 = self.shift_register,
                0xE000..=0xFFFF => self.prg_bank = self.shift_register,
                _ => panic!("cant be"),
            }

            self.reset();
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::VERTICAL,   // ? one-screen, lower bank
            1 => Mirroring::HORIZONTAL, // ? onescreen, upper bank
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
            _ => panic!("not support mirroring mode."),
        }
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        // prg_ramは6000から始まる
        self.prg_ram[addr as usize - 0x6000] = data;

        //保存処理
        let mut file = File::create(self.rom.save_data_file.as_str()).unwrap();
        file.write_all(&self.prg_ram).unwrap();
        file.flush().unwrap();
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        // フラグ設定
        match addr {
            // 気球入手済み
            0x628E => {
                return 0x02;
            }
            // 勇者のステータス最強
            0x6007..=0x600B => {
                return 0xFF;
            }
            // 敵一体目のHPが0
            0x727E..=0x727F => {
                return 0x00;
            }
            _ => {}
        }
        self.prg_ram[addr as usize - 0x6000]
    }

    fn load_prg_ram(&mut self, raw: &Vec<u8>) {
        if raw.is_empty() {
            return;
        }
        self.prg_ram = raw.to_vec()
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        // デバッグモード
        if addr == 0xC000 {
            return 0x01;
        }

        let bank_size = 16 * 1024 as usize; //16kB
        let bank_max = self.rom.prg_rom.len() / bank_size;
        let mut bank = self.prg_bank & 0x0F;
        let mut first_bank = 0x00;
        let mut last_bank = bank_max - 1;

        //chr_bankの5ビット目が立っていない場合
        if self.chr_bank0 & 0x10 != 0 {
            bank = bank | 0x10;
            first_bank = first_bank | 0x10;
            last_bank = last_bank | 0x10;
        } else {
            bank = bank & 0x0F;
            first_bank = first_bank & 0x0F;
            last_bank = last_bank & 0x0F;
        }

        match (self.control & 0x0C) >> 2 {
            0 | 1 => {
                // バンク番号の下位ビットを無視して32kBを$8000に切り替える
                self.rom.prg_rom
                    [((addr as usize - 0x8000) + bank_size * (bank & 0x1E) as usize) as usize]
            }

            2 => {
                //最初のバンクを$8000に固定し16kBバンクを$C000に切り替える
                match addr {
                    0x8000..=0xBFFF => {
                        self.rom.prg_rom[addr as usize - 0x8000 + bank_size * first_bank]
                    }
                    0xC000..=0xFFFF => {
                        self.rom.prg_rom
                            [((addr as usize - 0xC000) + bank_size * bank as usize) as usize]
                    }
                    _ => panic!("cant be"),
                }
            }
            3 => {
                //最初のバンクを$C000に固定し16kBバンクを$8000に切り替える
                match addr {
                    0x8000..=0xBFFF => {
                        self.rom.prg_rom
                            [((addr as usize - 0x8000) + bank_size * bank as usize) as usize]
                    }
                    0xC000..=0xFFFF => {
                        self.rom.prg_rom[addr as usize - 0xC000 + bank_size * last_bank]
                    }
                    _ => {
                        println!("addr: {:04X}", addr);
                        panic!("cant be")
                    }
                }
            }
            _ => panic!("cant be"),
        }
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
    fn scanline(&mut self, _scanline: usize, _show_background: bool) {}
    fn is_irq(&mut self) -> bool {
        false
    }
}

pub struct Mapper2 {
    pub rom: Rom,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 {
            rom: Rom::empty(),
            bank_select: 0,
        }
    }
}

impl Mapper for Mapper2 {
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom;
    }
    fn write(&mut self, _addr: u16, data: u8) {
        self.bank_select = data;
    }

    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn write_prg_ram(&mut self, _addr: u16, _data: u8) {}
    fn read_prg_ram(&self, _addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, _raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_size = 16 * 1024 as usize; //16kB
        let bank_max = self.rom.prg_rom.len() / bank_size;
        match addr {
            0x8000..=0xBFFF => {
                //bank select
                //addr - 0x8000で0からのスタートにする
                let bank = self.bank_select & 0x0F; //下位4ビットを取ってくる
                self.rom.prg_rom[((addr as usize - 0x8000) + bank_size * bank as usize) as usize]
            }
            0xC000..=0xFFFF => {
                // fix last bank
                self.rom.prg_rom[((addr as usize - 0xC000) + bank_size * (bank_max - 1)) as usize]
            }
            _ => panic!("cant be"),
        }
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
    fn scanline(&mut self, _scanline: usize, _show_background: bool) {}
    fn is_irq(&mut self) -> bool {
        false
    }
}

pub struct Mapper3 {
    pub rom: Rom,
    bank_select: u8,
}

impl Mapper3 {
    pub fn new() -> Self {
        Mapper3 {
            rom: Rom::empty(),
            bank_select: 0,
        }
    }
}

impl Mapper for Mapper3 {
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom;
    }
    fn write(&mut self, _addr: u16, data: u8) {
        self.bank_select = data;
    }

    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn write_prg_ram(&mut self, _addr: u16, _data: u8) {}
    fn read_prg_ram(&self, _addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, _raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.rom.prg_rom[addr as usize - 0x8000]
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        let bank_size = 8 * 1024 as usize; //8kiB
        let bank = self.bank_select & 0x03; //最下位2bit
        self.rom.chr_rom[(addr as usize + bank_size * bank as usize) as usize]
    }
    fn scanline(&mut self, _scanline: usize, _show_background: bool) {}
    fn is_irq(&mut self) -> bool {
        false
    }
}

pub struct Mapper4 {
    pub rom: Rom,
    prg_ram: Vec<u8>,
    bank_select: u8,
    bank_data: [u8; 8],
    mirroring: u8,
    prg_ram_protect: u8,
    irq_latch: u8,
    irq_latch_counter: u8,
    irq_reload: bool,
    irq_enable: bool,
    is_irq: bool,
}

impl Mapper4 {
    pub fn new() -> Self {
        Mapper4 {
            rom: Rom::empty(),
            prg_ram: vec![0xFF; 8192], //8kiB
            bank_select: 0,
            bank_data: [0; 8],
            mirroring: 0,
            prg_ram_protect: 0,
            irq_latch: 0,
            irq_latch_counter: 0,
            irq_reload: false,
            irq_enable: false,
            is_irq: false,
        }
    }
    fn get_chr_rom_addr(&self, addr: u16) -> usize {
        let bank_size = 1 * 1024 as usize; //1kiB

        let mode = self.bank_select & 0x80;
        let r0_bank = self.bank_data[0] as usize;
        let r1_bank = self.bank_data[1] as usize;
        let r2_bank = self.bank_data[2] as usize;
        let r3_bank = self.bank_data[3] as usize;
        let r4_bank = self.bank_data[4] as usize;
        let r5_bank = self.bank_data[5] as usize;

        match mode {
            0 => match addr {
                // 0x0000..=0x03FF
                // 0x0400..=0x07FF
                0x0000..=0x07FF => addr as usize + r0_bank * bank_size,
                // 0x0800..=0x0BFF
                // 0x0C00..=0x0FFF
                0x0800..=0x0FFF => (addr as usize - 0x0800) + r1_bank * bank_size,
                0x1000..=0x13FF => (addr as usize - 0x1000) + r2_bank * bank_size,
                0x1400..=0x17FF => (addr as usize - 0x1400) + r3_bank * bank_size,
                0x1800..=0x1BFF => (addr as usize - 0x1800) + r4_bank * bank_size,
                0x1C00..=0x1FFF => (addr as usize - 0x1C00) + r5_bank * bank_size,
                _ => panic!("cant be"),
            },
            _ => match addr {
                0x0000..=0x03FF => addr as usize + r2_bank * bank_size,
                0x0400..=0x07FF => (addr as usize - 0x0400) + r3_bank * bank_size,
                0x0800..=0x0BFF => (addr as usize - 0x0800) + r4_bank * bank_size,
                0x0C00..=0x0FFF => (addr as usize - 0x0C00) + r5_bank * bank_size,
                // 0x1000..=0x13FF
                // 0x1400..=0x17FF
                0x1000..=0x17FF => (addr as usize - 0x1000) + r0_bank * bank_size,
                // 0x1800..=0x1BFF
                // 0x1C00..=0x1FFF
                0x1800..=0x1FFF => (addr as usize - 0x1800) + r1_bank * bank_size,
                _ => panic!("cant be"),
            },
        }
    }
}

impl Mapper for Mapper4 {
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn set_rom(&mut self, rom: Rom) {
        self.load_prg_ram(&rom.save_data);
        self.rom = rom;
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => {
                if addr & 0x0001 == 0 {
                    //偶数のとき
                    // バンクセレクト
                    self.bank_select = data;
                } else {
                    // バンクデータ R0~R7
                    self.bank_data[(self.bank_select & 0x07) as usize] = data;
                }
            }

            0xA000..=0xBFFF => {
                if addr & 0x0001 == 0 {
                    // ミラーリング
                    self.mirroring = data;
                } else {
                    // PRG RAM 保護
                    self.prg_ram_protect = data;
                }
            }

            0xC000..=0xDFFF => {
                if addr & 0x0001 == 0 {
                    // IRQ ラッチ
                    self.irq_latch = data;
                } else {
                    // IRQ リロード
                    self.irq_reload = true;
                    self.irq_latch_counter = 0;
                }
            }

            0xE000..=0xFFFF => {
                if addr & 0x0001 == 0 {
                    // IRQ 無効化
                    self.irq_enable = false;
                    self.is_irq = false;
                } else {
                    // IRQ enable
                    self.irq_enable = true;
                }
            }

            _ => panic!("cant be"),
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.mirroring & 0x01 == 0 {
            Mirroring::VERTICAL
        } else {
            Mirroring::HORIZONTAL
        }
    }
    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        // prg_ramは6000から始まる
        self.prg_ram[addr as usize - 0x6000] = data;

        //保存処理
        let mut file = File::create(self.rom.save_data_file.as_str()).unwrap();
        file.write_all(&self.prg_ram).unwrap();
        file.flush().unwrap();
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize - 0x6000]
    }

    fn load_prg_ram(&mut self, raw: &Vec<u8>) {
        if raw.is_empty() {
            return;
        }
        self.prg_ram = raw.to_vec()
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_size = 8 * 1024 as usize; //8kiB
        let bank_max = (self.rom.prg_rom.len() / bank_size) as usize;

        let mode = self.bank_select & 0x40;
        let last_bank = bank_max - 1;
        let last_bank2 = bank_max - 2;
        let r6_bank = self.bank_data[6] as usize;
        let r7_bank = self.bank_data[7] as usize;

        match mode {
            0 => match addr {
                // R6, R7, (-2), (-1)
                0x8000..=0x9FFF => {
                    self.rom.prg_rom[((addr - 0x8000) as usize + r6_bank * bank_size) as usize]
                }
                0xA000..=0xBFFF => {
                    self.rom.prg_rom[((addr - 0xA000) as usize + r7_bank * bank_size) as usize]
                }
                0xC000..=0xDFFF => {
                    self.rom.prg_rom[((addr - 0xC000) as usize + last_bank2 * bank_size) as usize]
                }
                0xE000..=0xFFFF => {
                    self.rom.prg_rom[((addr - 0xE000) as usize + last_bank * bank_size) as usize]
                }
                _ => panic!("cant be"),
            },
            _ => match addr {
                // (-2), R7, R6, (-1)
                0x8000..=0x9FFF => {
                    self.rom.prg_rom[((addr - 0x8000) as usize + last_bank2 * bank_size) as usize]
                }
                0xA000..=0xBFFF => {
                    self.rom.prg_rom[((addr - 0xA000) as usize + r7_bank * bank_size) as usize]
                }
                0xC000..=0xDFFF => {
                    self.rom.prg_rom[((addr - 0xC000) as usize + r6_bank * bank_size) as usize]
                }
                0xE000..=0xFFFF => {
                    self.rom.prg_rom[((addr - 0xE000) as usize + last_bank * bank_size) as usize]
                }
                _ => panic!("cant be"),
            },
        }
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        let mirror_addr = self.get_chr_rom_addr(addr) as usize;
        self.rom.chr_rom[mirror_addr] = value;
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[self.get_chr_rom_addr(addr) as usize]
    }

    fn scanline(&mut self, scanline: usize, show_bkgd_or_sprt: bool) {
        if scanline <= 240 && show_bkgd_or_sprt {
            if self.irq_latch_counter == 0 || self.irq_reload {
                self.irq_latch_counter = self.irq_latch;
                self.irq_reload = false;
            } else {
                self.irq_latch_counter -= 1;
            }

            if self.irq_latch_counter == 0 && self.irq_enable {
                self.is_irq = true;
            }
        }
    }

    fn is_irq(&mut self) -> bool {
        let res = self.is_irq;
        self.is_irq = false;
        res
    }
}
