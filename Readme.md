### install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup --version
rustc --version
cargo --version
### rustプロジェクトファイルの作成
cargo new file_name

cargo install cargo-edit
### パッケージの追加
cargo add <パッケージ名>
cargo add <パッケージ名>@<バージョン指定>

### パッケージの追加(開発用)
cargo add <パッケージ名> --dev

### パッケージのアップグレード
cargo upgrade <パッケージ名>

### パッケージの削除
cargo rm <パッケージ名>

cargo install cargo-watch
rustup component add rustfmt
rustup component add clippy
rustup component add rls rust-src rust-analysis

### 安定版のみアップデートする
rustup update stable

### nightlyビルドのみアップデートする
rustup update nightly

### すべてアップデートする
rustup update

### sdl2
cargo add sdl2

### linuxにも入れる
sudo apt-get install libsdl2-dev
<!-- sudo apt-get install libsdl2-image-dev libsdl2-mixer-dev libsdl2-net-dev libsdl2-ttf-dev -->

error: XDG_RUNTIME_DIR is invalid or not set in the environment.
が出たら
export XDG_RUNTIME_DIR=/run/user/<user_id>
<user_id>は
$ id
で見ることができる

### rand
cargo add rand


### Xserver install
https://sourceforge.net/projects/vcxsrv/
ここからVcXsrvをインストール
するとXlaunchが入る. 色々設定し完了までする
cmdでipconfigを実行するとwslのIPv4アドレスが分かる
export DISPLAY=<WindowsのIPアドレス>:0.0
を実行
DISPLAY環境変数にWindowsマシンのIPアドレスを設定し、X Server経由でGUIアプリケーションを表示するための設定
wslを念のためアップデート
cmdで　wsl --update

### コンパイル、実行
cargo run

テスト
cargo test

ビルド
cargo build
