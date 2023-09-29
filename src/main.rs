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
                // 0x~~�͂P�U�i���\�L�@�@0b~~�͂Q�i���\�L
                0xA9 => {
                    let param = program[self.program_counter as usize];
                    self/program_counter += 1;
                    self.register_a = param;

                    // zero flag
                    if self.register_a == 0 {
                        //1bit�ځi������Q�ځAZ�j�ɂP�����Bor�Ŏ���1bit���K��1�����B
                        self.status = self.status | 0b0000_0010;
                    } else {
                        //zero flag���O����Ȃ��ꍇ�A1bit�ڂ��O�ɂ��āA����ȊO�͂��̂܂܁B
                        self.status = self.status & 0b1111_1101;
                    }

                    // negative flag
                    // 7bit�ڂ��P�̂Ƃ��inegative flag�������Ă���Ƃ��j
                    if self.register_a & 0b1000_0000 != 0 {
                        // 7bit�ڂɂP�𗧂ĂĂ���ȊO�͂��̂܂܁B
                        self.status = self.status | 0b1000_0000;
                    } else {
                        // 7bit�ڂ��O�ɂ��đ��͂��̂܂܁B
                        self.status = self.status & 0b0111_1111;
                    }
                }

                 _ => todo!("")
            }
        }
        todo!("");
    }
}