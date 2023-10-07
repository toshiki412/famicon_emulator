fn main() { 
    println!("Hello, world!");
}

#[derive(Debug, PartialEq)]     //deriveは継承。Debugトレイトを継承。{:?}が使用可能など。
#[allow(non_camel_case_types)]  //allowはリンカに対してnon_camel_case_typeのエラーを出さないようにする

pub enum AddressingMode {
    Accumulator,
    Immidiate,  //PCの値をそのままアキュムレータ(a)に入れる。           LDA #$10 => A9 10     aに10を入れる
    ZeroPage,   //アドレスの指す先の値を入れる                         LDA $10 => A5 10      aに0x10にある値を入れる
    ZeroPage_X, //アドレス+xレジスタのアドレスの指す先の値を入れる       LDA $44,X => B5 44    aに0x44 + x 番目のアドレスにある値を入れる
    ZeroPage_Y, //yレジスタバージョン　　                              LDX $44,Y => B6 44
    Absolute,   //直接アドレスを指定してそのアドレスの指す先の値を入れる。LDA $4400 => AD 00 44  aに0x4400にある値を入れる
    Absolute_X, //absoluteにxレジスタの値分足したアドレスを指定する。   LDA $4400,X => BD 00 44
    Absolute_Y, //yレジスタバージョン　　                              LDA $4400,Y => B9 00 44
    Indirect_X,
    Indirect_Y,
    Relative,
    Implied,
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
            AddressingMode::Accumulator => {
                panic!("AddressingMode::Accumulator");
            }
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
            AddressingMode::Relative => {
                let base = self.mem_read(self.program_counter);
                let np = (base as i8) as i32 + self.program_counter as i32;
                np as u16
            }
            AddressingMode::Implied => {
                panic!("AddressingMode Implied");
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
                    // self.brk(&AddressingMode::Implied);
                    break;
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

                // AND
                0x29 => {
                    self.and(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }

                // EOR
                0x49 => {
                    self.eor(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }

                // ORA
                0x09 => {
                    self.ora(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }

                // ASL
                0x06 => {
                    self.asl(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                0x0A => {
                    self.asl(&AddressingMode::Accumulator);
                }

                // LSR
                0x46 => {
                    self.lsr(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                0x4A => {
                    self.lsr(&AddressingMode::Accumulator);
                }

                // ROL
                0x26 => {
                    self.rol(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                0x2A => {
                    self.rol(&AddressingMode::Accumulator);
                }

                // ROR
                0x66 => {
                    self.ror(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                0x6A => {
                    self.ror(&AddressingMode::Accumulator);
                }

                // BCC
                0x90 => {
                    self.bcc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BCS
                0xB0 => {
                    self.bcs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }                

                // BEQ
                0xF0 => {
                    self.beq(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BNE
                0xD0 => {
                    self.bne(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BIT
                0x24 => {
                    self.bit(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }

                // BMI
                0x30 => {
                    self.bmi(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BPL
                0x10 => {
                    self.bpl(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // BVC
                0x50 => {
                    self.bvc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                // BVS
                0x70 => {
                    self.bvs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }

                // CLC
                0x18 => {
                    self.clc(&AddressingMode::Implied);
                }

                // SEC
                0x38 => {
                    self.sec(&AddressingMode::Implied);
                }

                // CLD
                0xd8 => {
                    self.cld(&AddressingMode::Implied);
                }

                // SED
                0xf8 => {
                    self.sed(&AddressingMode::Implied);
                }

                // CLI
                0x58 => {
                    self.cli(&AddressingMode::Implied);
                }

                // SEI
                0x78 => {
                    self.sei(&AddressingMode::Implied);
                }

                // CLV
                0xB8 => {
                    self.clv(&AddressingMode::Implied);
                }

                // CMP
                0xC9 => {
                    self.cmp(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }

                // CPX
                0xE0 => {
                    self.cpx(&AddressingMode::Immidiate);
                    self.program_counter += 1;
                }

                // CPY
                0xC0 => {
                    self.cpy(&AddressingMode::Immidiate);
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

    // AND
    // A,Z,N = A&M
    fn and(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);  //memory
        let value = self.mem_read(addr);            //memoryの値

        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // EOR
    // A,Z,N = A^M  exclusive or
    fn eor(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);  //memory
        let value = self.mem_read(addr);            //memoryの値

        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // ORA
    // A,Z,N = A|M 
    fn ora(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);  //memory
        let value = self.mem_read(addr);            //memoryの値

        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // ASL
    // A,Z,C,N = A*2  or M,Z,C,N = M*2
    fn asl(&mut self, mode:&AddressingMode){
        let (value,carry) = if mode == &AddressingMode::Accumulator{
            let (value,carry) = self.register_a.overflowing_mul(2);
            self.register_a = value;
            (value,carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            self.mem_write(addr,value);
            (value,carry)
        };

        self.status = if carry { 
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    // LSR
    // A,Z,C,N = A/2  or M,Z,C,N = M/2
    fn lsr(&mut self, mode: &AddressingMode){
        let (value,carry) = if mode == &AddressingMode::Accumulator{
            let carry = self.register_a & 0x01; 
            self.register_a = self.register_a / 2;
            (self.register_a,carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            self.mem_write(addr,value);
            (value,carry)
        };

        self.status = if carry == 1 { 
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    // ROL
    // rotate.  10101100 -> 01011001
    fn rol(&mut self, mode:&AddressingMode){
        let (value,carry) = if mode == &AddressingMode::Accumulator{
            let (value,carry) = self.register_a.overflowing_mul(2);
            self.register_a = value | (self.status & FLAG_CARRY);  //2倍するので下位1bitは絶対0
            (self.register_a,carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            let value = value | (self.status & FLAG_CARRY);
            self.mem_write(addr,value | (self.status & FLAG_CARRY));
            (value,carry)
        };

        self.status = if carry { 
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    // ROR
    // rotate right. 11110000 -> 01111000
    fn ror(&mut self, mode: &AddressingMode){
        let (value,carry) = if mode == &AddressingMode::Accumulator{
            let carry = self.register_a & 0x01; 
            self.register_a = self.register_a / 2;
            self.register_a = self.register_a | ((self.status & FLAG_CARRY) << 7); //7bitずらす
            (self.register_a,carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            let value = value | ((self.status & FLAG_CARRY) << 7);
            self.mem_write(addr,value);
            (value,carry)
        };

        self.status = if carry == 1 { 
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    // BCC 
    // carry flagが立っていないとき、分岐する
    fn bcc(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_CARRY, false);
    }

    // BCS 
    // carry flagが立っているとき、分岐する
    fn bcs(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_CARRY, true);
    }

    // BEQ 
    // zero flagが立っているとき、分岐する
    fn beq(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_ZERO, true);
    }

    // BNE 
    // zero flagが立っていないとき、分岐する
    fn bne(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_ZERO, false);
    }

    // BIT
    // A&M, N=M7, V=M6
    fn bit(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let zero = self.register_a & value;
        if zero == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }
        let flags = FLAG_NEGATIVE | FLAG_OVERFLOW;
        self.status = (self.status & !flags) | (value & flags);
    }

    // BMI
    // negative flagが立っているとき、分岐する
    fn bmi(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_NEGATIVE, true);
    }

    // BPL
    // negative flagが立っていないとき、分岐する
    fn bpl(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_NEGATIVE, false);
    }

    // BVS
    // overflow flagが立っているとき、分岐する
    fn bvs(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_OVERFLOW, true);
    }

    // BVC
    // overflow flagが立っていないとき、分岐する
    fn bvc(&mut self, mode: &AddressingMode){
        self._branch(mode, FLAG_OVERFLOW, false);
    }

    // branch
    fn _branch(&mut self, mode: &AddressingMode, flag: u8, is_flag: bool){
        let addr = self.get_operand_address(mode);
        if is_flag {
            if self.status & flag != 0 { //flagが立っているとき
                self.program_counter = addr
            }
        } else {
            if self.status & flag == 0 { //flagが立っていないとき
                self.program_counter = addr
            }
        }
    }

    // BRK
    fn brk(&mut self, _mode: &AddressingMode){
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    // CLC
    fn clc(&mut self, _mode: &AddressingMode){
        self.status = self.status & !FLAG_CARRY;
    }

    // SEC
    fn sec(&mut self, _mode: &AddressingMode){
        self.status = self.status | FLAG_CARRY;
    }

    // CLD
    fn cld(&mut self, _mode: &AddressingMode){
        self.status = self.status & !FLAG_DECIMAL;
    }

    // SED
    fn sed(&mut self, _mode: &AddressingMode){
        self.status = self.status | FLAG_DECIMAL;
    }

    // CLI
    fn cli(&mut self, _mode: &AddressingMode){
        self.status = self.status & !FLAG_INTERRUCT;
    }

    // SEI
    fn sei(&mut self, _mode: &AddressingMode){
        self.status = self.status | FLAG_INTERRUCT;
    }

    // CLV
    fn clv(&mut self, _mode: &AddressingMode){
        self.status = self.status & !FLAG_OVERFLOW;
    }

    // CMP
    // if A>=M carry flag set.  if A=M zero flag set.
    fn cmp(&mut self, mode: &AddressingMode){
        self._compare(self.register_a, mode);
    }

    // CPX
    // if X>=M carry flag set.  if X=M zero flag set.
    fn cpx(&mut self, mode: &AddressingMode){
        self._compare(self.register_x, mode);
    }

    // CPY
    // if Y>=M carry flag set.  if Y=M zero flag set.
    fn cpy(&mut self, mode: &AddressingMode){
        self._compare(self.register_y, mode);
    }

    // compare
    fn _compare(&mut self, target: u8, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if target >= value {
            self.sec(&AddressingMode::Implied);
        } else {
            self.clc(&AddressingMode::Implied);
        }

        let (value, _ ) = target.overflowing_sub(value);
        self.update_zero_and_negative_flags(value);
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

    fn assert_status(cpu: &CPU, flags: u8) {
        assert_eq!(cpu.status, flags)
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let cpu = run(vec![0xa9, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_a, 0x05);
        assert_status(&cpu,0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let cpu = run(vec![0xa9, 0x00, 0x00], |_| {});
        assert_status(&cpu,FLAG_ZERO);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let cpu = run(vec![0xa9, 0x80, 0x00], |_| {});
        assert_status(&cpu, FLAG_NEGATIVE);
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

    // ADC
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

    // SBC
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
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // AND
    #[test]
    fn test_and() {
        let cpu = run(vec![0x29,0x01,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    //EOR
    #[test]
    fn test_eor() {
        let cpu = run(vec![0x49,0x01,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(&cpu, 0);
    }

    // ORA
    #[test]
    fn test_ora() {
        let cpu = run(vec![0x09,0x01,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x03);
        assert_status(&cpu, 0);
    }

    // ASL
    #[test]
    fn test_asl_accumulator() {
        let cpu = run(vec![0x0a,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_asl_accumulator_occur_carry() {
        let cpu = run(vec![0x0a,0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_asl_zero_page() {
        let cpu = run(vec![0x06,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_asl_zero_page_occur_carry() {
        let cpu = run(vec![0x06,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x81);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    // LSR
    #[test]
    fn test_lsr_accumulator() {
        let cpu = run(vec![0x4a,0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_accumulator_occur_carry() {
        let cpu = run(vec![0x4a,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01); // 3 / 2 = 1, carry
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_lsr_zero_page() {
        let cpu = run(vec![0x46,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_zero_page_zero_flag() {
        let cpu = run(vec![0x46,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x01);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x00);
        assert_status(&cpu, FLAG_ZERO | FLAG_CARRY);
    }

    #[test]
    fn test_lsr_zero_page_occur_carry() {
        let cpu = run(vec![0x46,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x3);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    // ROL
    #[test]
    fn test_rol_accumulator() {
        let cpu = run(vec![0x2a,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_accumulator_with_carry() {
        let cpu = run(vec![0x2a,0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page() {
        let cpu = run(vec![0x26,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_with_carry() {
        let cpu = run(vec![0x26,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_accumulator_is_zero_with_carry() {
        let cpu = run(vec![0x2a,0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_zero_with_carry() {
        let cpu = run(vec![0x26,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    // ROR
    #[test]
    fn test_ror_accumulator() {
        let cpu = run(vec![0x6a,0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_accumulator_occur_carry() {
        let cpu = run(vec![0x6a,0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_zero_page() {
        let cpu = run(vec![0x66,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_zero_page_occur_carry() {
        let cpu = run(vec![0x66,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x3);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_accumulator_with_carry() {
        let cpu = run(vec![0x6a,0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_with_carry() {
        let cpu = run(vec![0x66,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x3);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_accumulator_is_zero_with_carry() {
        let cpu = run(vec![0x6a,0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_zero_with_carry() {
        let cpu = run(vec![0x66,0x01,0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // BCC 
    #[test]
    fn test_bcc() {
        let cpu = run(vec![0x90,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01); //0x90 0x02より2命令進んで0xe8に到達
        assert_eq!(cpu.program_counter, 0x8006); //PCは8000から始まって6命令進んでる
        assert_status(&cpu,0);
    }

    #[test]
    fn test_bcc_with_carry() {
        let cpu = run(vec![0x90,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_CARRY; //carryが立っている場合、分岐はしない
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu,FLAG_CARRY);
    }

    #[test]
    fn test_bcc_negative() {
        let cpu = run(vec![0x90,0xfc,0x00], |cpu| { //0xfc = -4
            cpu.mem_write(0x7FFF, 0x00);
            cpu.mem_write(0x7FFE, 0xe8);
            //この段階で 0xe8 0x00 0x90 0xfc 0x00という命令列になっている
            //90読んでfcから4つ前に戻るのでe8に到達
        }); 
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8000);
        assert_status(&cpu,0);
    }

    // BCS 
    #[test]
    fn test_bcs() {
        let cpu = run(vec![0xb0,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu,0);
    }

    #[test]
    fn test_bcs_with_carry() {
        let cpu = run(vec![0xb0,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu,FLAG_CARRY);
    }

    // BEQ
    #[test]
    fn test_beq() {
        let cpu = run(vec![0xf0,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu,0);
    }

    #[test]
    fn test_beq_with_zero_flag() {
        let cpu = run(vec![0xf0,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_ZERO; //zeroが立っている場合、分岐する
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu,0); //INXよりzero flagが落ちる
    }

    // BNE
    #[test]
    fn test_bne() {
        let cpu = run(vec![0xd0,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.program_counter, 0x8006);
        assert_status(&cpu,0);
    }

    #[test]
    fn test_bne_with_zero_flag() {
        let cpu = run(vec![0xd0,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_ZERO; //zeroが立っている場合、分岐しない
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.program_counter, 0x8003);
        assert_status(&cpu,FLAG_ZERO);
    }

    // BIT
    #[test]
    fn test_bit() {
        let cpu = run(vec![0x24,0x00,0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x0000, 0x00);
        });
        assert_status(&cpu, FLAG_ZERO);
    }

    #[test]
    fn test_bit_negative_flag() {
        let cpu = run(vec![0x24,0x00,0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x0000, 0x80);
        });
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_ZERO);
    }

    #[test]
    fn test_bit_overflow_flag() {
        let cpu = run(vec![0x24,0x00,0x00], |cpu| {
            cpu.register_a = 0x40;
            cpu.mem_write(0x0000, 0x40);
        });
        assert_status(&cpu, FLAG_OVERFLOW);
    }

    // BMI
    #[test]
    fn test_bmi() {
        let cpu = run(vec![0x30,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8003);
    }

    #[test]
    fn test_bmi_with_negative_flag() {
        let cpu = run(vec![0x30,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_eq!(cpu.program_counter, 0x8006);
        assert_eq!(cpu.register_x, 0x01); 
        assert_status(&cpu, 0); // INXよりnegativeが落ちる
    }

    // BPL
    #[test]
    fn test_bpl() {
        let cpu = run(vec![0x10,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.program_counter, 0x8006);
        assert_eq!(cpu.register_x, 0x01); 
        assert_status(&cpu, 0); // INXよりnegativeが落ちる
    }

    #[test]
    fn test_bpl_with_negative_flag() {
        let cpu = run(vec![0x10,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_NEGATIVE);
        assert_eq!(cpu.program_counter, 0x8003);
    }

    // BVC
    #[test]
    fn test_bvc() {
        let cpu = run(vec![0x50,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.program_counter, 0x8006);
        assert_eq!(cpu.register_x, 0x01); 
        assert_status(&cpu, 0); // INXよりnegativeが落ちる
    }

    #[test]
    fn test_bvc_with_overflow_flag() {
        let cpu = run(vec![0x50,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_OVERFLOW);
        assert_eq!(cpu.program_counter, 0x8003);
    }

    // BVS
    #[test]
    fn test_bvs() {
        let cpu = run(vec![0x70,0x02,0x00,0x00,0xe8,0x00], |_| {});
        assert_eq!(cpu.program_counter, 0x8003);
        assert_eq!(cpu.register_x, 0x00); 
        assert_status(&cpu, 0); 
    }

    #[test]
    fn test_bvs_with_overflow_flag() {
        let cpu = run(vec![0x70,0x02,0x00,0x00,0xe8,0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, FLAG_OVERFLOW);
        assert_eq!(cpu.program_counter, 0x8006);
    }

    // CLC
    #[test]
    fn test_clc(){
        let cpu = run(vec![0x18,0x00], |cpu| {
            cpu.status = FLAG_CARRY | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SEC
    #[test]
    fn test_sec(){
        let cpu = run(vec![0x38,0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    // CLD
    #[test]
    fn test_cld(){
        let cpu = run(vec![0xd8,0x00], |cpu| {
            cpu.status = FLAG_DECIMAL | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SED
    #[test]
    fn test_sed(){
        let cpu = run(vec![0xf8,0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_DECIMAL | FLAG_NEGATIVE);
    }

    // CLI
    #[test]
    fn test_cli(){
        let cpu = run(vec![0x58,0x00], |cpu| {
            cpu.status = FLAG_INTERRUCT | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SEI
    #[test]
    fn test_sei(){
        let cpu = run(vec![0x78,0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_INTERRUCT | FLAG_NEGATIVE);
    }

    // CLV
    #[test]
    fn test_clv(){
        let cpu = run(vec![0xb8,0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // CMP
    #[test]
    fn test_cmp(){
        let cpu = run(vec![0xc9,0x01], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY); //2>=1よりキャリーが立つ
    }

    #[test]
    fn test_cmp_eq(){
        let cpu = run(vec![0xc9,0x02], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO); //2=2よりキャリーとゼロが立つ
    }

    #[test]
    fn test_cmp_no_flag(){
        let cpu = run(vec![0xc9,0x03], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_NEGATIVE); //2<3 よりnegativeが立つ (2-3より)
    }

    // CPX
    #[test]
    fn test_cpx(){
        let cpu = run(vec![0xe0,0x01], |cpu| {
            cpu.register_x = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY);
    }

    // CPY
    #[test]
    fn test_cpy(){
        let cpu = run(vec![0xc0,0x01], |cpu| {
            cpu.register_y = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY);
    }
    
}