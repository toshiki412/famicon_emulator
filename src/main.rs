fn main() { 
    println!("Hello, world!");
}

pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opscode {
                // 0x~~は１６進数表記　　0b~~は２進数表記
                0xA9 => {
                    let param = program[self.program_counter as usize];
                    self/program_counter += 1;
                    self.register_a = param;

                    // zero flag
                    if self.register_a == 0 {
                        //1bit目（左から２個目、Z）に１が立つ。orで取ると1bitが必ず1が立つ。
                        self.status = self.status | 0b0000_0010;
                    } else {
                        //zero flagが０じゃない場合、1bit目を０にして、それ以外はそのまま。
                        self.status = self.status & 0b1111_1101;
                    }

                    // negative flag
                    // 7bit目が１のとき（negative flagが立っているとき）
                    if self.register_a & 0b1000_0000 != 0 {
                        // 7bit目に１を立ててそれ以外はそのまま。
                        self.status = self.status | 0b1000_0000;
                    } else {
                        // 7bit目を０にして他はそのまま。
                        self.status = self.status & 0b0111_1111;
                    }
                }

                 _ => todo!("")
            }
        }
        todo!("");
    }
}