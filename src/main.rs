#[macro_use] //マクロはメインに書かなきゃいけない
extern crate lazy_static;

//モジュールのインポートはメインに書かなきゃいけない
mod cpu;
mod opscodes;

fn main() {
    println!("Hello, world!");
}
