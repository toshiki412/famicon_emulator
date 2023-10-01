fn main() { 
    println!("Hello, world!");
}

#[derive(Debug)]                //deriveは継承。Debugトレイトを継承。{:?}が使用可能など。
#[allow(non_camel_case_types)]  //allowはリンカに対してnon_camel_case_typeのエラーを出さないようにする

pub enum AddressingMode {
    Immidiate,  //PCの値をそのままアキュムレータ(a)に入れる。           LDA #$10 => A9 10     aに10を入れる
    ZeroPage,   //アドレスの指す先の値を入れる                         LDA $10 => A5 10      aに0x10にある値を入れる
    ZeroPage_X, //アドレス+xレジスタのアドレスの指す先の値を入れる       LDA $44,X => B5 44    aに0x44 + x 番目のアドレスにある値を入れる
    ZeroPage_Y, //yレジスタバージョン　　                              LDX $44,Y => B6 44
    Absolute,   //直接アドレスを指定してそのアドレスの指す先の値を入れる。LDA $4400 => AD 00 44  aに0x4400にある値を入れる
    Absolute_X, //absoluteにxレジスタの値分足したアドレスを指定する。   LDA $4400,X => BD 00 44
    Absolute_Y, //yレジスタバージョン　　                              LDA $4400,Y => B9 00 44
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct CPU {
    pub register_a: u8,         //アキュムレータ
    pub register_x: u8,         //汎用レジスタ
    pub register_y: u8,         //汎用レジスタ
    pub status: u8,             //フラグレジスタ
    pub program_counter: u16,   //プログラムカウンタ。現在実行中の命令のアドレスを示す。
    memory: [u8; 0x10000], //FIX 0xFFFF to 0x10000
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0x10000], 
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immidiate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                //wrapping_addはオーバーフローの制御。
                //posにwrapping_addしてregister_xを足した結果のaddrが
                //オーバーフローしてもエラーは出ずに切り捨てられた数値が入る
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
            
        }
    }

    //指定したアドレス(addr)から1バイト(8bit)のデータを読む関数
    pub fn mem_read(&self, addr: u16) -> u8{
        self.memory[addr as usize]  //usizeは32bit or 64bitの符号なし整数。プラットフォームに依存しない。
    }

    //指定したアドレス(pos)から2バイト(16bit)のデータを読む関数
    pub fn mem_read_u16(&self, pos: u16)  -> u16{
        let lo = self.mem_read(pos) as u16;     //アドレスposから1バイト目のデータを取得
        let hi = self.mem_read(pos+1) as u16;   //アドレスpos+1から2バイト目のデータを取得
        (hi << 8) | (lo as u16)                 //取得した二つのバイトを結合して返す
    }

    //指定したアドレス(addr)に1バイトのデータを書き込む
    pub fn mem_write(&mut self, addr: u16, data: u8){
        self.memory[addr as usize] = data;
    }

    //指定したアドレス(pos)に2バイトのデータを書き込む
    pub fn mem_write_u16(&mut self, pos: u16, data: u16){
        //16bitのデータ(data)を8bitのハイバイトとローバイトに分ける
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;

        self.mem_write(pos, lo);
        self.mem_write(pos+1,hi);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>){
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>){
        //8000番地から上にカートリッジ（ファミコンのカセット、プログラム）のデータを書き込む
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn reset(&mut self){
        self.register_a = 0;
        self.register_x = 0; 
        self.register_y = 0; 
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }


    pub fn run(&mut self){
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                // LDA
                0xA9 => {
                    self.lda(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;  //absoluteは後ろに2バイトあるので+2
                }
                0xBD => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                // BRK
                0x00 => {
                    return;
                }

                // TAX
                0xAA => {
                    self.tax();
                }

                // INX
                0xE8 => {
                    self.inx();
                }

                // STA
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                 _ => todo!("")
            }
        }
        // todo!("");
    }


    // LDA immidiate
    // A,Z,N = M  アキュムレータにメモリにある値をロードする
    fn lda(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // TAX
    // x = A   xレジスタにアキュムレータの値を代入
    fn tax(&mut self){
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // INX
    // X,Z,N = X+1  xレジスタに１を足す
    fn inx(&mut self){
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // STA
    // M = A   メモリにアキュムレータの値をストアする
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8){
        // zero flag
        if result == 0 {
            //1bit目(左から2個目、Z)に1が立つ。orで取ると1bitが必ず1が立つ。
            // self.status = self.status | 0b0000_0010;
            self.status = self.status | 0x02;
        } else {
            //zero flagが０じゃない場合、1bit目を０にして、それ以外はそのまま
            // self.status = self.status & 0b1111_1101;
            self.status = self.status & 0xFD;
        }

        // negative flag
        // 7bit目が１のとき（negative flagが立っているとき）
        // if result & 0b1000_0000 != 0 
        if result & 0x80 != 0 {
            // 7bit目に１を立ててそれ以外はそのまま。
            // self.status = self.status | 0b1000_0000;
            self.status = self.status | 0x80;
        } else {
            // 7bit目を０にして他はそのまま。
            // self.status = self.status & 0b0111_1111;
            self.status = self.status & 0x7F;
        }
    }
}

#[cfg(test)]    //cfgは条件付きコンパイル。テストするとき以外はこのモジュールはコンパイルしない
mod test {
    use super::*;

    // 0x~~は16進数表記 0b~~は2進数表記

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu: CPU = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        // assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0x02 == 0x00);
        // assert!(cpu.status & 0b1000_0000 == 0);
        assert!(cpu.status & 0x80 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu: CPU = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        // assert!(cpu.status & 0b0000_0010 == 0b10);
        assert!(cpu.status & 0x02 == 0x02);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu: CPU = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]);
        // assert!(cpu.status & 0b1000_0000 != 0);
        assert!(cpu.status & 0x80 == 0x80);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xaa, 0x00]);
        cpu.reset();
        // cpu.register_a = 10;
        cpu.register_a = 0x0A;
        cpu.run();
        assert_eq!(cpu.register_x, 0x0A);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu: CPU = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        // 0xa9, 0xc0でアキュムレータにに0xc0をロード
        // 0xaaでxレジスタにアキュムレータの値を代入
        // 0xe8でxレジスタの値を1だけインクリメント
        // 0x00でbreak
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow(){
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_lda_from_memory_zero_page() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xa5,0x10,0x00]);
        cpu.reset();
        cpu.mem_write(0x10,0x55);               //メモリの0x10番地に0x55を書き込む
        cpu.run();
        assert_eq!(cpu.register_a, 0x55)
    }

    #[test]
    fn test_lda_from_memory_zero_page_x() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xb5,0x10,0x00]);
        cpu.reset();
        cpu.mem_write(0x11,0x56);
        cpu.register_x = 0x01;
        cpu.run();
        assert_eq!(cpu.register_a, 0x56);
    }

    #[test]
    fn test_lda_from_memory_absolute() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xad,0x10,0xaa,0x00]);
        cpu.reset();
        cpu.mem_write(0xaa10,0x57);
        cpu.run();
        assert_eq!(cpu.register_a, 0x57);
    }

    #[test]
    fn test_lda_from_memory_absolute_x() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xbd,0x10,0xaa,0x00]);
        cpu.reset();
        cpu.mem_write(0xaa15,0x58);
        cpu.register_x = 0x05;
        cpu.run();
        assert_eq!(cpu.register_a, 0x58);
    }

    #[test]
    fn test_lda_from_memory_absolute_y() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xb9,0x10,0xaa,0x00]);
        cpu.reset();
        cpu.mem_write(0xaa18,0x59);
        cpu.register_y = 0x08;
        cpu.run();
        assert_eq!(cpu.register_a, 0x59);
    }

    #[test]
    fn test_lda_from_memory_indirect_x() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xa1,0x10,0x00]);
        cpu.reset();
        cpu.mem_write(0x18,0x05);   //0x10にregisterxを足した番地を読む(low byte)
        cpu.mem_write(0x19,0xff);   //その次の番地(0x19)も読む(high byte)
        cpu.mem_write(0xff05,0x5a); //lowとhighを結合した番地に値を書き込んでおく(これがアキュムレータにロードされる)
        cpu.register_x = 0x08;
        cpu.run();
        assert_eq!(cpu.register_a, 0x5a); //二つの番地から取ったものを足した番地のところの値をアキュムレータがロードする
    }

    #[test]
    fn test_lda_from_memory_indirect_y() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0xb1,0x10,0x00]);
        cpu.reset();
        cpu.mem_write(0x10,0x06);   //0x10番地を読む(low byte)
        cpu.mem_write(0x11,0xff);   //その次の番地(0x11)も読む(high byte)
        cpu.mem_write(0xff09,0x5b); //lowとhighを結合した番地にyを足した番地に値を書き込んでおく(これがアキュムレータにロードされる)
        cpu.register_y = 0x03;
        cpu.run();
        assert_eq!(cpu.register_a, 0x5b); //結合した番地の値をアキュムレータがロードする
    }

    #[test]
    fn test_sta_from_memory() {
        let mut cpu: CPU = CPU::new();
        cpu.load(vec![0x85,0x10,0x00]);
        cpu.reset();
        cpu.register_a = 0xba;
        cpu.run();
        assert_eq!(cpu.mem_read(0x10), 0xba);
    }
}