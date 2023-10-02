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

const FLAG_CARRY:u8 = 1;            //0b00000001, 0x01
const FLAG_ZERO:u8 = 1 << 1;        //0b00000010, 0x02
const FLAG_INTERRUCT:u8 = 1 << 2;   //0b00000100, 0x04
const FLAG_DECIMAL:u8 = 1 << 3;     //0b00001000, 0x08
const FLAG_BREAK:u8 = 1 << 4;       //0b00010000, 0x10
const FLAG_OVERFLOW:u8 = 1 << 6;    //0b01000000, 0x40
const FLAG_NEGATIVE:u8 = 1 << 7;    //0b10000000, 0x80

const SIGN_BIT:u8 = 1 << 7;

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

                // ADC
                0x69 => {
                    self.adc(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }


                // SBC
                0xE9 => {
                    self.sbc(&AddressingMode::Immidiate);
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

    // ADC
    // A,Z,C,N = A+M+C
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);  //memory
        let value = self.mem_read(addr);            //memoryの値

        let carry = self.status & FLAG_CARRY;
        let (rhs, carry_flag1) = value.overflowing_add(carry);     //桁溢れが生じたらflagがtrue
        let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

        let overflow = (self.register_a & SIGN_BIT) == (value & SIGN_BIT) 
                       && (value & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if carry_flag1 || carry_flag2 { //どちらかのキャリーフラグが立っている場合
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a);

    }

    // SBC
    // A,Z,C,N = A-M-(1-C)
    fn sbc(&mut self, mode: &AddressingMode) {
        //キャリーかどうかの判定が逆
        //overflowの判定が逆  minus.plusかplus.minus

        let addr = self.get_operand_address(mode);  //memory
        let value = self.mem_read(addr);            //memoryの値

        let carry = self.status & FLAG_CARRY;
        let (v1, carry_flag1) = self.register_a.overflowing_sub(value); //A-M
        let (n, carry_flag2) = v1.overflowing_sub(1-carry);

        let overflow = (self.register_a & SIGN_BIT) != (value & SIGN_BIT) 
                       && (self.register_a & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if !(carry_flag1 || carry_flag2) { 
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };

        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a);

    }

    fn update_zero_and_negative_flags(&mut self, result: u8){
        // zero flag
        if result == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }

        // negative flag
        if result & 0x80 != 0 {
            self.status = self.status | FLAG_NEGATIVE;
        } else {
            self.status = self.status & !FLAG_NEGATIVE;
        }
    }
}

#[cfg(test)]    //cfgは条件付きコンパイル。テストするとき以外はこのモジュールはコンパイルしない
mod test {
    use super::*;

    fn run<F>(program: Vec<u8>, f:F) -> CPU
    where
        F: Fn(&mut CPU),
        {
            let mut cpu = CPU::new();
            cpu.load(program);
            cpu.reset();
            f(&mut cpu);
            cpu.run();

            cpu
        }

    fn assert_status(cpu: CPU, flags: u8) {
        assert_eq!(cpu.status, flags)
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let cpu = run(vec![0xa9, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_a, 0x05);
        assert_status(cpu,0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let cpu = run(vec![0xa9, 0x00, 0x00], |_| {});
        assert_status(cpu,FLAG_ZERO);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let cpu = run(vec![0xa9, 0x80, 0x00], |_| {});
        assert_status(cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let cpu = run(vec![0xaa, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_x, 0x0A);
    }

    #[test]
    fn test_5_ops_working_together() {
        //0xe8はinx
        let cpu = run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow(){
        let cpu = run(vec![0xe8, 0xe8, 0x00], |cpu| {
            cpu.register_x = 0xff;
        });
        assert_eq!(cpu.register_x, 0x01);
    }

    #[test]
    fn test_lda_from_memory_zero_page() {
        let cpu = run(vec![0xa5,0x10,0x00], |cpu| {
            cpu.mem_write(0x10,0x55);  //メモリの0x10番地に0x55を書き込む
        });
        assert_eq!(cpu.register_a, 0x55)
    }

    #[test]
    fn test_lda_from_memory_zero_page_x() {
        let cpu = run(vec![0xb5,0x10,0x00], |cpu| {
            cpu.mem_write(0x11,0x56);
            cpu.register_x = 0x01;
        });
        assert_eq!(cpu.register_a, 0x56);
    }

    #[test]
    fn test_lda_from_memory_absolute() {
        let cpu = run(vec![0xad,0x10,0xaa,0x00], |cpu| {
        cpu.mem_write(0xaa10,0x57);
        });
        assert_eq!(cpu.register_a, 0x57);
    }

    #[test]
    fn test_lda_from_memory_absolute_x() {
        let cpu = run(vec![0xbd,0x10,0xaa,0x00], |cpu| {
            cpu.mem_write(0xaa15,0x58);
            cpu.register_x = 0x05;
        });
        assert_eq!(cpu.register_a, 0x58);
    }

    #[test]
    fn test_lda_from_memory_absolute_y() {
        let cpu = run(vec![0xb9,0x10,0xaa,0x00], |cpu| {
            cpu.mem_write(0xaa18,0x59);
            cpu.register_y = 0x08;
        });
        assert_eq!(cpu.register_a, 0x59);
    }

    #[test]
    fn test_lda_from_memory_indirect_x() {
        let cpu = run(vec![0xa1,0x10,0x00], |cpu| {
            cpu.mem_write(0x18,0x05);   //0x10にregisterxを足した番地を読む(low byte)
            cpu.mem_write(0x19,0xff);   //その次の番地(0x19)も読む(high byte)
            cpu.mem_write(0xff05,0x5a); //lowとhighを結合した番地に値を書き込んでおく(これがアキュムレータにロードされる)
            cpu.register_x = 0x08;
        });
        assert_eq!(cpu.register_a, 0x5a); //二つの番地から取ったものを足した番地のところの値をアキュムレータがロードする
    }

    #[test]
    fn test_lda_from_memory_indirect_y() {
        let cpu = run(vec![0xb1,0x10,0x00], |cpu| {
            cpu.mem_write(0x10,0x06);   //0x10番地を読む(low byte)
            cpu.mem_write(0x11,0xff);   //その次の番地(0x11)も読む(high byte)
            cpu.mem_write(0xff09,0x5b); //lowとhighを結合した番地にyを足した番地に値を書き込んでおく(これがアキュムレータにロードされる)
            cpu.register_y = 0x03;
        });
        assert_eq!(cpu.register_a, 0x5b); //結合した番地の値をアキュムレータがロードする
    }

    #[test]
    fn test_sta_from_memory() {
        let cpu = run(vec![0x85,0x10,0x00], |cpu| {
            cpu.register_a = 0xba;
        });
        assert_eq!(cpu.mem_read(0x10), 0xba);
    }

    #[test]
    //carry flagが立ってない場合
    fn test_adc_no_carry() {
        let cpu = run(vec![0x69,0x10,0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    //carry flagが立っている場合
    fn test_adc_has_carry() {
        let cpu = run(vec![0x69,0x10,0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = 0x01;
        });
        assert_eq!(cpu.register_a, 0x31); //carryフラグの値も足すので0x30 + 0x01
        assert_eq!(cpu.status, 0x00);  //計算で桁あふれが生じないのでフラグは立たない
    }

    #[test]
    //carry flagが起こる場合
    fn test_adc_occur_carry() {
        let cpu = run(vec![0x69,0x02,0x00], |cpu| {
            cpu.register_a = 0xff;
        });
        assert_eq!(cpu.register_a, 0x01); //0xff + 0x02で桁あふれが生じる
        assert_eq!(cpu.status, FLAG_CARRY);  //桁あふれが生じたのでcarryflagが立つ
    }

    #[test]
    //プラスの計算でoverflowが起こる場合
    fn test_adc_occur_overflow_plus() {
        let cpu = run(vec![0x69,0x10,0x00], |cpu| {
            cpu.register_a = 0x7f;       
        });
        assert_eq!(cpu.register_a, 0x8f); //00010000 + 01111111 = 10001111
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_NEGATIVE);  //正＋正で負のビットが立ったのでoverflowが立つ. negativeフラグも立つ。
    }

    #[test]
    //carryが立っていてプラスの計算でoverflowが起こる場合
    fn test_adc_occur_overflow_plus_with_carry() {
        let cpu = run(vec![0x69,0x6f,0x00], |cpu| {
            cpu.register_a = 0x10;
            cpu.status = 0x01;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_NEGATIVE);  //正＋正で負のビットが立ったのでoverflowが立つ. negativeフラグも立つ。
    }

    #[test]
    //マイナスの計算でoverflowが起こる場合
    fn test_adc_occur_overflow_minus() {
        let cpu = run(vec![0x69,0x81,0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02); //1000,0001 + 1000,0001 = 0000,0002 (1,0000,0002)
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_CARRY);  //負＋負で正になっているのでoverflowが立つ。桁もあふれたのでcarryも立つ
    }

    #[test]
    //carryが立っていてマイナスの計算でoverflowが起こる場合
    fn test_adc_occur_overflow_minus_with_carry() {
        let cpu = run(vec![0x69,0x80,0x00], |cpu| {
            cpu.register_a = 0x80;
            cpu.status = 0x01;
        });
        assert_eq!(cpu.register_a, 0x01); //1000,0000 + 1000,0000 = 0000,0000 これに1足す。
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_CARRY);  //overflowとcarryが立つ
    }

    #[test]
    //符号が違うものを足す場合
    fn test_adc_occur_no_overflow() {
        let cpu = run(vec![0x69,0x7f,0x00], |cpu| {
            cpu.register_a = 0x82;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, FLAG_CARRY);  //carryが立つ
    }


    #[test]
    fn test_sbc_no_carry() {
        let cpu = run(vec![0xe9,0x10,0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x0f); //c=0より1-c=1なので0x10から1引いて0x0f
        assert_eq!(cpu.status, FLAG_CARRY);  //carry判定じゃなければ立つので、立つ
    }


    #[test]
    fn test_sbc_has_carry() {
        let cpu = run(vec![0xe9,0x10,0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = 0x01;
        });
        assert_eq!(cpu.register_a, 0x10); 
        assert_eq!(cpu.status, FLAG_CARRY);  //carry判定じゃなければ立つので、立つ
    }

    #[test]
    fn test_sbc_occur_carry() {
        let cpu = run(vec![0xe9,0x02,0x00], |cpu| {
            cpu.register_a = 0x01;
        });
        assert_eq!(cpu.register_a, 0xfe); //0x01 - 0x02 - (0x01 - 0)
        assert_eq!(cpu.status, FLAG_NEGATIVE);
    }

    #[test]
    fn test_sbc_occur_overflow() {
        let cpu = run(vec![0xe9,0x81,0x00], |cpu| {
            cpu.register_a = 0x7f;
        });
        assert_eq!(cpu.register_a, 0xfd);
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_NEGATIVE); //negative flag, overflow
    }

    #[test]
    fn test_sbc_occur_overflow_with_carry() {
        let cpu = run(vec![0xe9,0x81,0x00], |cpu| {
            cpu.register_a = 0x7f;
            cpu.status = 0x01;
        });
        assert_eq!(cpu.register_a, 0xfe);
        assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_NEGATIVE); //negative flag, overflow
    }

    #[test]
    fn test_sbc_no_overflow() {
        let cpu = run(vec![0xe9,0x7f,0x00], |cpu| {
            cpu.register_a = 0x7e;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xff);
        assert_status(cpu, FLAG_NEGATIVE);
    }
}