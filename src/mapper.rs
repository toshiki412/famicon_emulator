pub struct Mapper2 {
    pub prg_rom: Vec<u8>,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 {
            prg_rom: vec![],
            bank_select: 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data;
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_len = 16 * 1024 as usize; //16kB
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x8000..=0xBFFF => {
                //bank select
                //addr - 0x8000で0からのスタートにする
                let bank = self.bank_select & 0x0F; //下位4ビットを取ってくる
                self.prg_rom[((addr as usize - 0x8000) + bank_len * bank as usize) as usize]
            }
            0xC000..=0xFFFF => {
                // fix last bank
                self.prg_rom[((addr as usize - 0xC000) + bank_len * (bank_max - 1)) as usize]
            }
            _ => panic!("cant be"),
        }
    }
}