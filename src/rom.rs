#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; //N E S ^Z
                                                   // const NES_TAG: Vec<u8> = vec![0x4E, 0x45, 0x53, 0x1A]; //N E S ^Z
const PRG_ROM_PAGE_SIZE: usize = 16 * 1024; // 16kiB
const CHR_ROM_PAGE_SIZE: usize = 8 * 1024; // 8kiB

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
    pub is_chr_ram: bool,

    pub save_data: Vec<u8>,
    pub save_data_file: String,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNes file format".to_string());
        }

        // Nes2.0のファイル形式の7番目と6番目のヘッダにマッパの情報がある
        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        // ex) raw[7] = 10101010, raw[6] = 11110000のとき
        // raw[7] & 0b1111_0000 => 1010_0000
        // raw[6] >> 4 => 0000_1111
        // mapper = 1010_1111

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let chr_rom = if chr_rom_size == 0 {
            //chr_rom_size = 0の場合、8KBのchr_romが存在する
            let blank_chr_ram: Vec<u8> = vec![0; CHR_ROM_PAGE_SIZE];
            blank_chr_ram
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: chr_rom,
            mapper: mapper,
            screen_mirroring: screen_mirroring,
            is_chr_ram: chr_rom_size == 0,
            save_data: Vec::new(),
            save_data_file: String::from(""),
        })
    }

    pub fn empty() -> Self {
        return Rom {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::VERTICAL,
            is_chr_ram: false,
            save_data: Vec::new(),
            save_data_file: String::from(""),
        };
    }
}
