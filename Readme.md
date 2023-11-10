### install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup --version
rustc --version
cargo --version
### rust�v���W�F�N�g�t�@�C���̍쐬
cargo new file_name

cargo install cargo-edit
### �p�b�P�[�W�̒ǉ�
cargo add <�p�b�P�[�W��>
cargo add <�p�b�P�[�W��>@<�o�[�W�����w��>

### �p�b�P�[�W�̒ǉ�(�J���p)
cargo add <�p�b�P�[�W��> --dev

### �p�b�P�[�W�̃A�b�v�O���[�h
cargo upgrade <�p�b�P�[�W��>

### �p�b�P�[�W�̍폜
cargo rm <�p�b�P�[�W��>

cargo install cargo-watch
rustup component add rustfmt
rustup component add clippy
rustup component add rls rust-src rust-analysis

### ����ł̂݃A�b�v�f�[�g����
rustup update stable

### nightly�r���h�̂݃A�b�v�f�[�g����
rustup update nightly

### ���ׂăA�b�v�f�[�g����
rustup update

### sdl2
cargo add sdl2

### linux�ɂ������
sudo apt-get install libsdl2-dev
<!-- sudo apt-get install libsdl2-image-dev libsdl2-mixer-dev libsdl2-net-dev libsdl2-ttf-dev -->

error: XDG_RUNTIME_DIR is invalid or not set in the environment.
���o����
export XDG_RUNTIME_DIR=/run/user/<user_id>
<user_id>��
$ id
�Ō��邱�Ƃ��ł���

### rand
cargo add rand


### Xserver install
https://sourceforge.net/projects/vcxsrv/
��������VcXsrv���C���X�g�[��
�����Xlaunch������. �F�X�ݒ肵�����܂ł���
cmd��ipconfig�����s�����wsl��IPv4�A�h���X��������
export DISPLAY=<Windows��IP�A�h���X>:0.0
�����s
DISPLAY���ϐ���Windows�}�V����IP�A�h���X��ݒ肵�AX Server�o�R��GUI�A�v���P�[�V������\�����邽�߂̐ݒ�
wsl��O�̂��߃A�b�v�f�[�g
cmd�Ł@wsl --update

### �R���p�C���A���s
cargo run

�e�X�g
cargo test

�r���h
cargo build
