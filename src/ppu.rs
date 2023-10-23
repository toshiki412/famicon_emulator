use crate::rom::Mirroring;
use bitflags::bitflags;

pub struct NesPPU {
    pub chr_rom: Vec<u8>,        //背景やキャラの画像データ
    pub palette_table: [u8; 32], //色の情報
    pub vram: [u8; 2048],
    pub mirroring: Mirroring,
    cycles: usize,
    scanline: usize,
    pub nmi_interrupt: Option<i32>,
    internal_data_buf: u8,

    pub ctrl: ControlRegister,  //0x2000 割り込みなどPPUの設定 write
    mask: MaskRegister,         //0x2001 拝啓enableなどのPPUの設定 write
    status: StatusRegister,     //0x2002 PPUのステータス read
    pub oam_addr: u8,           //0x2003 書き込むスプライト領域のアドレス write
    pub oam_data: [u8; 256],    //0x2004 スプライト領域のデータ read/write
    pub scroll: ScrollRegister, //0x2005 scroll write
    addr: AddrRegister, //0x2006(0x2007) 書き込むPPUメモリ領域のアドレス、データ (0x2006 write, 0x2007 read/write)
}

impl NesPPU {
    // chr_romがカートリッジ(ゲーム)から直接接続されてくるrom
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_addr: 0,
            oam_data: [0; 64 * 4],
            mirroring: mirroring,
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
            status: StatusRegister::new(),
            mask: MaskRegister::new(),
            scroll: ScrollRegister::new(),
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
        match addr {
            0..=0x1FFF => {
                // FIXME
            }
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => {
                // FIXME
            }
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    //0x2000のコントロールレジスタへの書き込み
    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn read_status(&mut self) -> u8 {
        self.scroll.reset();
        self.status.bits()
    }

    pub fn write_to_status(&mut self, value: u8) {
        self.status.update(value);
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_dma(&mut self, values: [u8; 256]) {
        self.oam_data = values;
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.set(value);
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
            if self.is_sprite_zero_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            //0~262lineのうち241~は画面外
            if self.scanline == 241 {
                self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    // self.status.set_vblank_status(true);
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
                self.nmi_interrupt = None;
                return true;
            }

            if self.scanline == 257 {
                self.oam_addr = 0;
            }
        }
        return false;
    }

    fn is_sprite_zero_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
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
        *self.0.bits_mut() = data;
    }

    pub fn generate_vblank_nmi(&mut self) -> bool {
        let last_status = self.contains(ControlRegister::GENERATE_NMI);
        self.set(ControlRegister::GENERATE_NMI, true);
        return last_status;
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    // pub fn is_sprite_8x16_mode(&self) -> bool {
    //     self.contains(ControlRegister::SPRITE_SIZE)
    // }

    pub fn sprite_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn name_table_addr(&self) -> u16 {
        match (
            self.contains(ControlRegister::NAMETABLE2),
            self.contains(ControlRegister::NAMETABLE1),
        ) {
            (false, false) => 0x2000,
            (false, true) => 0x2400,
            (true, false) => 0x2800,
            (true, true) => 0x2C00,
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

    pub fn set_sprite_zero_hit(&mut self, value: bool) {
        self.set(StatusRegister::SPRITE_ZERO_HIT, value)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

bitflags! {
    pub struct MaskRegister: u8 {
        const GRAYSCALE                 = 0b0000_0001;
        const SHOW_BACKGROUND_IN_LEFT   = 0b0000_0010;
        const SHOW_SPRITES_IN_LEFT      = 0b0000_0100;
        const SHOW_BACKGROUND           = 0b0000_1000;
        const SHOW_SPRITES              = 0b0001_0000;
        const EMPHASIZE_RED             = 0b0010_0000;
        const EMPHASIZE_GREEN           = 0b0100_0000;
        const EMPHASIZE_BLUE            = 0b1000_0000;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SHOW_SPRITES)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    write_x: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            write_x: false,
        }
    }

    fn set(&mut self, data: u8) {
        if self.write_x {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.write_x = !self.write_x;
    }

    //一回しかLDAしなかったときのためにリセットをかける
    pub fn reset(&mut self) {
        self.write_x = true;
    }
}
