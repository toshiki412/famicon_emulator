use crate::rom::{Mirroring, Rom};
use std::{
    fs::File,
    io::{self, BufReader, Write},
};

pub fn create_mapper(rom: Rom) -> Box<dyn Mapper> {
    let mut mapper: Box<dyn Mapper> = match rom.mapper {
        0 => Box::new(Mapper0::new()),
        1 => Box::new(Mapper1::new()),
        2 => Box::new(Mapper2::new()),
        3 => Box::new(Mapper3::new()),
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
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
            _ => panic!("not support mirroring mode."),
        }
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        self.prg_ram[addr as usize - 0x6000] = data;

        //FIXME 保存処理
        let mut file = File::create("save.dat").unwrap();
        file.write_all(&self.prg_ram).unwrap();
        file.flush().unwrap();
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize - 0x6000]
    }

    fn load_prg_ram(&mut self, raw: &Vec<u8>) {
        self.prg_ram = raw.to_vec()
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
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
        self.rom.prg_rom[(addr as usize - 0x8000)]
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        let bank_size = 8 * 1024 as usize; //8kiB
        let bank = self.bank_select & 0x03; //最下位2bit
        self.rom.chr_rom[(addr as usize + bank_size * bank as usize) as usize]
    }
}
