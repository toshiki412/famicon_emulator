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
    Implicit,
    NoneAddressing,
}

struct OpCode {
    code: u8,
    mnemonic: String,
    bytes: u8,
    cycles: u8,
    addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        mnemonic: &str,
        bytes: u8,
        cycles: u8,
        addressing_mode: AddressingMode,
    ) -> Self {
        OpCode {
            code: code,
            mnemonic: String::from(mnemonic),
            bytes: bytes,
            cycles: cycles,
            addressing_mode: addressing_mode,
        }
    }
}