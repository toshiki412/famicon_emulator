use std::fs::File;
use std::io::Read;

use crate::rom::Rom;

pub fn load_rom(path: &str) -> Rom {
    let mut f = File::open(path).expect("no file found");
    let metadata = std::fs::metadata(path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let rom = Rom::new(&buffer).expect("load error");
    rom
}


pub mod test {    
    use super::*; //外側の関数とuseを全部持ってくる

    // pub fn snake_rom() -> Rom {
    //     load_rom("rom/snake.nes")
    // }

    pub fn test_rom() -> Rom {
        load_rom("rom/nestest.nes")
    }
}