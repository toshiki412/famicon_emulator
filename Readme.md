install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup --version
rustc --version
cargo --version
# rust�v���W�F�N�g�t�@�C���̍쐬
cargo new file_name

cargo install cargo-edit
# �p�b�P�[�W�̒ǉ�
cargo add <�p�b�P�[�W��>
cargo add <�p�b�P�[�W��>@<�o�[�W�����w��>

# �p�b�P�[�W�̒ǉ�(�J���p)
cargo add <�p�b�P�[�W��> --dev

# �p�b�P�[�W�̃A�b�v�O���[�h
cargo upgrade <�p�b�P�[�W��>

# �p�b�P�[�W�̍폜
cargo rm <�p�b�P�[�W��>

cargo install cargo-watch
rustup component add rustfmt
rustup component add clippy
rustup component add rls rust-src rust-analysis

# ����ł̂݃A�b�v�f�[�g����
rustup update stable

# nightly�r���h�̂݃A�b�v�f�[�g����
rustup update nightly

# ���ׂăA�b�v�f�[�g����
rustup update

# sdl2
cargo add sdl2

# linux�ɂ������
sudo apt-get install libsdl2-dev
<!-- sudo apt-get install libsdl2-image-dev libsdl2-mixer-dev libsdl2-net-dev libsdl2-ttf-dev -->

error: XDG_RUNTIME_DIR is invalid or not set in the environment.
���o����
export XDG_RUNTIME_DIR=/run/user/<user_id>
<user_id>��
$ id
�Ō��邱�Ƃ��ł���

# rand
cargo add rand


# Xserver install
https://sourceforge.net/projects/vcxsrv/
��������VcXsrv���C���X�g�[��
�����Xlaunch������. �F�X�ݒ肵�����܂ł���
cmd��ipconfig�����s�����wsl��IPv4�A�h���X��������
export DISPLAY=<Windows��IP�A�h���X>:0.0
�����s
DISPLAY���ϐ���Windows�}�V����IP�A�h���X��ݒ肵�AX Server�o�R��GUI�A�v���P�[�V������\�����邽�߂̐ݒ�
wsl��O�̂��߃A�b�v�f�[�g
cmd�Ł@wsl --update

# �R���p�C���A���s
cargo run

�e�X�g
cargo test

�r���h
cargo build

# ����

## �t���O���W�X�^
NV-BDIZC

N Negative flag : ���Z���ʂ�7bit�ځi�ŏ�ʃr�b�g�j���P�Ȃ�Z�b�g
V Overflow flag : �v���X���m���邢�̓}�C�i�X���m�̉��Z�ŕ������ς�����Ƃ��ɃZ�b�g
- ��ɂP���Z�b�g
B Break command : BRK���߂��s��ꂽ���ƂɃZ�b�g
D Decimal mode  : ���ꂪ�Z�b�g����Ă���ƃv���Z�b�T�͉��Z�ƌ��Z�̍ۂɃo�C�i���Z�p�ɏ]��
I Interrupt disalbe : SEI���߂̌�ɃZ�b�g�B�Z�b�g����Ă���ԃv���Z�b�T�̓f�o�C�X����̊��荞�݂ɉ������Ȃ�
Z Zero flag     : ���Z���ʂ�0�̂Ƃ��Z�b�g
C Carry flag    : ���Z���ʂōŏ�ʃr�b�g���I�[�o�[�t���[�܂��͍ŉ��ʃr�b�g���A���_�[�t���[�����Ƃ��ɃZ�b�g


## �y�[�W
0x0000 ~ 0x00FF 255�� 1�y�[�W�@���ɂ���0x0000~�̂P�y�[�W���[���y�[�W�Ƃ���
���̃y�[�W�͗D������Ă��đ����A�N�Z�X�ł���

## unwrap()
Option��Result�Ƃ������񋓌^�ienums�j�������ۂɎg�p�����֗��ȃ��\�b�h�̈��.

- Option<T>�F����́A����l�����݂��邩�ǂ�����\�����邽�߂Ɏg�p����܂��B�l�����݂���ꍇ�ASome(T)�Ƃ��ĕ\������A�l�����݂��Ȃ��ꍇ�ANone�Ƃ��ĕ\������܂��B

- Result<T, E>�F����́A�֐����������邩���s���邩��\�����邽�߂Ɏg�p����܂��B��������ꍇ�AOk(T)�Ƃ��ĕ\������A���s����ꍇ�AErr(E)�Ƃ��ĕ\������܂��B

unwrap()���\�b�h�͂����̗񋓌^����l�����o�����߂Ɏg���܂�.
Option�ɑ΂��ĂȂ�Some�̒l�AResult�ɑ΂��ĂȂ�Ok�̏ꍇ�̒��̒l�����o��

ex)
let some_value: Option<i32> = Some(42);
let unwrapped = some_value.unwrap();

## event_pump(), poll_iter()
- �C�x���g�|���v�́A�C�x���g����������ꏊ��\�[�X(sdl_context�Ȃ�)����C�x���g�����W���A�v���O�����ɒ񋟂��邽�߂̒��ۉ��ł��B�ʏ�A�E�B���h�E�V�X�e������̃��[�U�[���́i�L�[�{�[�h��}�E�X�̃C�x���g�Ȃǁj��A���̑��̊O���C�x���g�i�t�@�C���V�X�e���̕ύX�A�l�b�g���[�N�C�x���g�Ȃǁj����M���A�v���O�������ŏ����\�Ȍ`���ɕϊ����܂��B

- poll_iter()�́A�C�x���g�|���v����擾�ł���C�x���g�𔽕��������邽�߂̃��\�b�h�ł��B���̃��\�b�h�́A���̂悤�Ȃ��Ƃ��s���܂��F
1 �C�x���g�|���v���Ǘ�����C�x���g�L���[���玟�̃C�x���g���擾���܂��B
2 �擾�����C�x���g���v���O�������ŏ������邽�߂ɒ񋟂��܂��B
3 �C�x���g���������ꂽ�ꍇ�A�L���[����폜����܂��B

�ʏ�A���̃��\�b�h�𖳌����[�v���Ŏg�p���āA���[�U�[����̓��͂�V�X�e���C�x���g�����A���^�C���ɊĎ����A����ɉ����ăA�v���P�[�V�����̐U�镑����ύX���܂��B


## CPU Memory Map
[0x0000...0x1FFF] CPU RAM
- [0x0000...0x07FF]��WRAM, [0x0800...0x1FFF]��WRAM�̃~���[�����O
- [0x0100...0x00FF]�̓X�^�b�N�̈�Ƃ��Ďg�p����

[0x2000...0x3FFF] IO Port PPU 
- PPU���W�X�^[0x2000...0x2007] ����̓A�h���X���[0x2008...0x3FFF]�Ƀ~���[�����O�����

[0x4000...0x401F] IO Port APU, Joypad

[0x4020...0x5FFF] Expansion Rom 
- Mapper. �J�[�g���b�W�̐���ɂ���ĈقȂ���@�Ŏg�p�����X�y�[�X

[0x6000...0x7FFF] Save RAM (�g��RAM)
- �J�[�g���b�W�� RAM �X�y�[�X������ꍇ�A�J�[�g���b�W��� RAM �X�y�[�X�\�񂳂��. �Q�[���̏�Ԃ�ۑ�������Ȃ�.

[0x8000....0xFFFF] Prg ROM(�J�[�g���b�W��̃v���O����ROM�X�y�[�X��Ƀ}�b�v�����)
- [0x8000....0xBFFF]��prg rom��LOW, [0xC000...0xFFFF]��prg rom��HIGH
- NMI [0xFFFA,0xFFFB], RESET [0xFFFC,0xFFFD], IRQ/BRK [0xFFFE,0xFFFF]



## PPU�iPicture Processing Unit�j

VRAM�ւ̃A�N�Z�X

�A�h���X ( 0x2006 ) �ƃf�[�^ ( 0x2007 )���G�~�����[�g����ꍇ
1.�v���A�h���X�� Addr ���W�X�^�Ƀ��[�h����K�v������܂��B2 �o�C�g�� 1 �o�C�g�̃��W�X�^�Ƀ��[�h����ɂ́A���W�X�^�� 2 �񏑂����ޕK�v������܂�

2.���ɁACPU �� PPU �f�[�^ ���W�X�^ (0x2007) ����̃f�[�^ ���[�h��v���ł��܂�

3.�ŏI�I�� PPU �̓����o�b�t�@����l���擾����ɂ́ACPU �� 0x2007 ���������x�ǂݎ��K�v������܂�

write[0x2006] = 0x60
write[0x2006] = 0x00
read[0x2007]  <- PPU�͒l�������Ԃ��Ȃ��̂ň��ڂ̓_�~�[��Ԃ�
PPU��cha_rom��read[0x0600]����
chr_rom����f�[�^���󂯎��X�g�A����
�C���N�������g����
CPU����read[0x2007]�@�i2��ځj
PPU�̓X�g�A�����l��Ԃ�

## PPU Memory Map
PPU��6502����̓}�b�v����Ă��炸�A�f�[�^�̑���M��I/O�|�[�g��ʂ��čs��

[0x0000...0x0FFF] �p�^�[���e�[�u��LOW
[0x0100...0x1FFF] �p�^�[���e�[�u��HIGH

[0x2000...0x23BF] ���1 �l�[���e�[�u��
[0x23C0...0x23FF] ���1 �����e�[�u��
[0x2400...0x27BF] ���2 �l�[���e�[�u��
[0x27C0...0x27FF] ���2 �����e�[�u��

[0x2800...0x2BBF] ���3 �l�[���e�[�u��
- [0x2000...0x23BF]�̃~���[�����O

[0x28C0...0x2BFF] ���3 �����e�[�u��
- [0x23C0...0x23FF]�̃~���[�����O

[0x2C00...0x2FBF] ���4 �l�[���e�[�u��
- [0x2400...0x27BF]�̃~���[�����O

[0x2FC0...0x2FFF] ���4 �����e�[�u��
- [0x27C0...0x27FF]�̃~���[�����O

[0x3000...0x3EFF] ���g�p
- [0x2000...0x2EFF]�̃~���[�����O

[0x3F00...0x3F0F] BG�p���b�g�e�[�u��
[0x3F10...0x3F1F] �X�v���C�g�p���b�g�e�[�u��

[0x3F20...0x3FFF] ���g�p
- [0x3F00...0x3F1F]�̃~���[�����O

[0x4000...0xFFFF] ���g�p
- [0x0000...0x3FFF]�̃~���[�����O


## Frame
�t���[��" �̓r�f�I�\���̒P�ʂ��w���܂��B��ʓI�ɁA�r�f�I�Q�[���̃O���t�B�b�N�X�͏u�Ԃ��Ƃɉ�ʂɕ\������܂��B���̏u�Ԃ� "�t���[��" �ƌĂт܂��B
�t�@�~�R���iNES�j�ɂ����āA1�̃t���[���͈ȉ��̗v�f����\������܂��F

1 ��ʕ\��: ��ʂɕ\�������r�f�I�O���t�B�b�N�X�A���Ȃ킿�Q�[���̔w�i�A�L�����N�^�[�A�I�u�W�F�N�g�A�e�L�X�g�Ȃǂ��܂܂�܂��B�����̗v�f��PPU�iPicture Processing Unit�j�ɂ���Đ�������A�e���r��ʂɕ\������܂��B

2 �Q�[�����W�b�N: �Q�[�����̃��W�b�N�A�v���C���[�̓��͂̏����A�L�����N�^�[�̓���A�Փ˔���Ȃǂ�1�̃t���[�����Ői�s���܂��B����ɂ��A�Q�[���̐i�s�󋵂����䂳��܂��B

3 �T�E���h�Ɖ��y: �Q�[�����̃T�E���h�G�t�F�N�g�≹�y��1�̃t���[�����Ő�������A�e���r�⃂�j�^�[�̃X�s�[�J�[����o�͂���܂��B����ɂ��A�Q�[���̃I�[�f�B�I�v�f���񋟂���܂��B

�t�@�~�R���iNES�j�̃Q�[���́A60�t���[��/�b�̃t���[�����[�g�ŕ\������邱�Ƃ���ʓI�ł����B


## render
PPU�����ʏ����擾���A�����frame�ɕ`�悷��
���ۂ̕`�揈�����s�����W���[��

## ��ʕ`��
- �^�C��
�^�C���́A��ʂ̏����Ȑ����`�̕`�惆�j�b�g�ł��B�ʏ�A8x8�s�N�Z���܂���8x16�s�N�Z���̐����`�ł��B
NES�̉�ʂ́A�����̃^�C�����z�u����Ăł��Ă���A�����̃^�C����g�ݍ��킹�ĉ�ʑS�̂�`�悵�܂��B
�^�C���͔w�i��X�v���C�g�i�L�����N�^�[�j�ȂǁA��ʏ�̂��܂��܂ȗv�f�̊�{�P�ʂƂ��Ďg�p����܂��B

- �^�C���}�b�v
�^�C���}�b�v�́A��ʏ�̂ǂ̈ʒu�ɂǂ̃^�C�����z�u����邩�������f�[�^�\���ł��B
�^�C���}�b�v�́A���W�n�Ɋ�Â��ĉ�ʏ�̊e�ʒu�ɑΉ�����^�C���ԍ��������A����ɂ���ĉ�ʂ��ǂ̂悤�ɕ`����邩�𐧌䂵�܂��B

- �p�^�[���e�[�u��
NES��PPU�́A�w�i�^�C���ƃX�v���C�g�^�C���̂��߂Ƀp�^�[���e�[�u���ƌĂ΂�郁�����̈�Ƀ^�C���̃O���t�B�b�N�X�f�[�^���i�[���܂��B
�p�^�[���e�[�u���ɂ́A�e�^�C���̃r�b�g�}�b�v���ۑ�����Ă���A�����ǂݎ���Ď��ۂ̕`��Ɏg�p���܂��B

- �l�[���e�[�u��
�l�[���e�[�u���́A�w�i�̃^�C���i�^�C���}�b�v�j�̔z�u�����i�[���郁�����̈�ł��B
�ʏ�ANES�̃l�[���e�[�u����2����A���ꂼ�ꂪ��ʂ̍������ƉE�����i�܂��͏㔼���Ɖ������j�ɑΉ����܂��B����́A�����~���[�����O�܂��͐����~���[�����O�̂��߂Ɏg�p����܂��B
�e�l�[���e�[�u���́A��ʏ�̊e�ʒu�ɑΉ�����^�C���̔ԍ��������Ă���A����ɂ���Ĕw�i�̕`���񂪊Ǘ�����܂��B
�Q�[���̃v���O���}�[�ƃf�U�C�i�[�́A�l�[���e�[�u����ҏW���āA�w�i�̔z�u��ݒ肵�܂��B

- �����e�[�u��
�����e�[�u���́A�l�[���e�[�u�����̃^�C���ɑΉ�����w�i�̐F�����Ǘ����܂��B
NES�̉�ʂ́A�w�i��8x8�s�N�Z���̃^�C���ō\������A�e�^�C���ɂ�16�F�̃p���b�g����I�������F�����蓖�Ă��܂��B�����e�[�u���́A�ǂ̃p���b�g���ǂ̃^�C���ɓK�p����邩�������܂��B
�ʏ�A1�̑����e�[�u����4�̃l�[���e�[�u���ɑΉ����A�l�[���e�[�u������16x16�^�C���O���b�h���Ƃɑ��������i�[���܂��B

�G�~�����[�^�́A�Q�[���J�[�g���b�W���̃f�[�^����l�[���e�[�u���Ƒ����e�[�u����ǂݎ��A�����̏������Ƃɉ�ʂ̔w�i�`����Č����܂��B

- �X�L�������C��
�r�f�I�f�B�X�v���C��J�����̉摜�����Ɋ֘A����p��ŁA���ɌÓT�I��CRT�i�A�ɐ��ǁj���j�^�[��e���r�ɂ����ďd�v�ȊT�O�ł��B�X�L�������C���́A��ʂ𐅕��ɑ�������ۂɈ�s���̉�f�i�s�N�Z���j����\���܂��B
��ʂ̕\���͒ʏ�A�����Ɉ�s����������܂��B����ɂ��A��ʑS�̂��`����܂��B�e�����s���u�X�L�������C���v�ƌĂ΂�A��ʑS�͕̂����̃X�L�������C������\������܂��B

- �~���[�����O
�ÓT�I�ȃQ�[���J�[�g���b�W�̃f�[�^�z�u�Ɋ֘A����T�O�ŁA��ɌÂ��Q�[���@�̃J�[�g���b�W���̃������\�����w���܂��B
�����~���[�����O�ł́APPU���������̈ꕔ�𕡐����A�������𕡐��̏ꏊ�ɔ��f�����܂��B
���̕����ł́A��ʂ̏㔼���Ɖ������̏�񂪓����ł��邽�߁A��������œ����f�[�^��2�̏ꏊ�ɕۑ����܂��B
�����~���[�����O�ł́APPU���������̈ꕔ�𔽉f�����A���������ł͓�����񂪕\������܂��B
���̕����ł́A��ʂ̍������ƉE�����̏�񂪓����ł��邽�߁A��������œ����f�[�^��2�̏ꏊ�ɕۑ����܂��B

�Q�[���J�[�g���b�W���̃������`�b�v�́A���̗e�ʂɉ����ăR�X�g��������܂��B�Q�[�����̃f�[�^�i�Q�[���v���O������Q�[���f�[�^�j�́A���̃������`�b�v�Ɋi�[����܂��B�������A�Q�[�����̃f�[�^�̒��ɂ͓������𕡐���g�p���邱�Ƃ����邽�߁A�����f�[�^�𕡐��̏ꏊ�ɔz�u�����̃f�[�^���قȂ�A�h���X����A�N�Z�X���邱�Ƃŉ�ʂɓ����f�[�^��\���ł��܂��B�܂�A�����f�[�^�𕡐��̏ꏊ�ɔz�u���邱�ƂȂ��A1�̃f�[�^�Z�b�g���ė��p���邱�Ƃ��ł���̂ł��B




## APU

�T�v
4ch �m�C�Y
3ch �O�p�g
1,2ch ��`�g

��`�g
�p�����[�^�F
- Duty��
- �{�����[��
- �w���c



2ch ��`�g

$4004 ���F�E����
DDLERRRR
R:���[�g
  - �G���x���[�v�L���Ȃ�G���x���[�v���x�̎w��A�����Ȃ特�ʂ̎w��imax15�j
E:�G���x���[�v�t���O
  - 0�Ȃ�L���A1�Ȃ疳��
L:�L�[�I�t�J�E���^�t���O�E�G���x���[�v���[�v�t���O
  - 0�ŃL�[�I�t�J�E���^�L���A�G���x���[�v���[�v����
  - 1�ŃL�[�I�t�J�E���^�����A�G���x���[�v���[�v�L��
D:Duty��
  - 00:12.5%  01:25%  10:50%  11:75%

$4005 �X�C�[�v
FSSSHRRR
R:���g���ύX��
H:���g���ύX����
S:���g���ύX�^�C�}�J�E���g��
F:�X�C�[�v�L���t���O

$4006 ���g������
FFFFFFFF
F:���g��

$4007 ���g����ʁE�L�[�I���I�t
CCCCCFFF
F:���g��
C:�L�[�I���J�E���g�l


3ch �O�p�g
$4008 ����
FLLLLLLL
L:����
F:�����E�L�[�I�t�J�E���^�L���t���O
  - 0�ŗL���A1�Ŗ���ςȂ�

$4009 ���g�p

$400A ���g������
FFFFFFFF
F:���g��

$400B ���g����ʁE�L�[�I���I�t
CCCCCFFF
F:���g��
C:�L�[�I���J�E���g�l


�m�C�Y

$400C�@����
XXLERRRR
X:���g�p
L:�L�[�I�t�J�E���^�t���O�E�G���x���[�v���[�v�t���O
  - 0�ŃL�[�I�t�J�E���^�L���A�G���x���[�v���[�v����
  - 1�ŃL�[�I�t�J�E���^�����A�G���x���[�v���[�v�L��
R:���[�g

$400D�@
���g�p

$400E�@���g��
RXXXFFFF
F:�m�C�Y���g��
X:���g�p
R:�������A�Z����
  - 0�Œ�����

$400F �L�[�I���E�I�t
CCCCCXXX
X:���g�p
C:�L�[�I�t�J�E���g�l
