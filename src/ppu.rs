use crate::rom::Mirroring;
use bitflags::bitflags;

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],

    pub mirroring: Mirroring,

    addr: AddrRegister,        //0x2006(0x2007)
    pub ctrl: ControlRegister, //0x2000
    internal_data_buf: u8,
    status: StatusRegister, //0x2002

    cycles: usize,
    scanline: usize,
    pub nmi_interrupt: Option<i32>,
}

impl NesPPU {
    // chr_romがカートリッジ(ゲーム)から直接接続されてくるrom
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            mirroring: mirroring,
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
            status: StatusRegister::new(),
            internal_data_buf: 0,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();
        self.vram[self.mirror_vram_addr(addr) as usize] = value;
    }

    //0x2000のコントロールレジスタへの書き込み
    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => panic!(
                "addr space 0x3000..0x3eff is not expected to be use, request = {}",
                addr
            ),
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        // mirror down 0x3000~0x3eff to 0x2000~0x2eff
        let mirrored_vram = addr & 0b10_1111_1111_1111;

        // to vram vector
        let vram_index = mirrored_vram - 0x2000;

        // to the name table index
        let name_table = vram_index / 0x400;

        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) => vram_index - 0x800,
            (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        //画面一列で341サイクル
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            //0~262lineのうち241~は画面外
            if self.scanline == 241 {
                if self.ctrl.generate_vblank_nmi() {
                    self.status.set_vblank_status(true);
                    // todo!("Should trigger NMI interrupt")
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.reset_vblank_status();
                return true;
            }
        }
        return false;
    }
}

pub struct AddrRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xFF) as u8;
    }

    //LDAで読み込む動作一回分
    //2回やるとloとhiのどちらにも書き込まれる
    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // ミラーリング
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);

        //桁上がりの処理
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        // ミラーリング
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111);
        }
    }

    //一回しかLDAしなかったときのためにリセットをかける
    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    // hiとloを一つにする
    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE1                = 0b0000_0001;
        const NAMETABLE2                = 0b0000_0010;
        const VRAM_ADD_INCREMENT        = 0b0000_0100;
        const SPRITE_PATTERN_ADDR       = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDR   = 0b0001_0000;
        const SPRITE_SIZE               = 0b0010_0000;
        const MASTER_SLAVE_SELECT       = 0b0100_0000;
        const GENERATE_NMI              = 0b1000_0000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        // self.bits = data;?
        *self.0.bits_mut() = data;
    }

    pub fn generate_vblank_nmi(&mut self) -> bool {
        let last_status = self.contains(ControlRegister::GENERATE_NMI);
        self.set(ControlRegister::GENERATE_NMI, true);
        last_status
    }

    pub fn bknd_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        const PPU_OPEN_BUS          = 0b0001_1111;
        const SPRITE_OVERFLOW       = 0b0010_0000;
        const SPRITE_ZERO_HIT       = 0b0100_0000;
        const VBLANK_HAS_STARTED    = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn is_in_vblank(&mut self) -> bool {
        self.contains(StatusRegister::VBLANK_HAS_STARTED)
    }

    pub fn set_vblank_status(&mut self, value: bool) {
        self.set(StatusRegister::VBLANK_HAS_STARTED, value)
    }

    pub fn reset_vblank_status(&mut self) {
        self.set_vblank_status(false)
    }
}
