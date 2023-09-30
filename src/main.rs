fn main() { 
    println!("Hello, world!");
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0xFFFF],
        }
    }

    //1バイト読む関数
    pub fn mem_read(&self, addr: u16) -> u8{
        self.memory[addr as usize]
    }

    pub fn mem_read_u16(&self, pos: u16)  -> u16{
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos+1) as u16;
        (hi << 8) | (lo as u16)
    }

    //指定の番地にデータを書き込む
    pub fn mem_write(&mut self, addr: u16, data: u8){
        self.memory[addr as usize] = data;
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16){
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

    pub fn reset(&mut self){
        // FIXME テストのために一旦コメントアウト
        // self.register_a = 0;
        // self.register_x = 0; 
        // self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>){
        //8000番地から上にカートリッジ（ファミコンのカセット、プログラム）のデータを書き込む
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self){
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                // 0x~~は16進数表記 0b~~は2進数表記
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(param);
                }

                // BRK
                0x00 => {
                    return;
                }

                //TAX
                0xAA => {
                    self.tax();
                }

                //INX
                0xE8 => {
                    self.inx();
                }

                 _ => todo!("")
            }
        }
        // todo!("");
    }


    // LDA immidiate
    // A,Z,N = M  アキュムレータにメモリをロードする
    fn lda(&mut self, value: u8){
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
        if self.register_x == 0xFF {
            self.register_x = 0;
        }else{
            self.register_x += 1;
        }

        self.update_zero_and_negative_flags(self.register_x);
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

#[cfg(test)]
mod test {
    use super::*;

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
        // cpu.register_a = 10;
        cpu.register_a = 0x0A;
        cpu.load_and_run(vec![0xaa, 0x00]);
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
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 1);
    }
}