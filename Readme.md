install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup --version
rustc --version
cargo --version
# rustプロジェクトファイルの作成
cargo new file_name

cargo install cargo-edit
# パッケージの追加
cargo add <パッケージ名>
cargo add <パッケージ名>@<バージョン指定>

# パッケージの追加(開発用)
cargo add <パッケージ名> --dev

# パッケージのアップグレード
cargo upgrade <パッケージ名>

# パッケージの削除
cargo rm <パッケージ名>

cargo install cargo-watch
rustup component add rustfmt
rustup component add clippy
rustup component add rls rust-src rust-analysis

# 安定版のみアップデートする
rustup update stable

# nightlyビルドのみアップデートする
rustup update nightly

# すべてアップデートする
rustup update

# sdl2
cargo add sdl2

# linuxにも入れる
sudo apt-get install libsdl2-dev
<!-- sudo apt-get install libsdl2-image-dev libsdl2-mixer-dev libsdl2-net-dev libsdl2-ttf-dev -->

error: XDG_RUNTIME_DIR is invalid or not set in the environment.
が出たら
export XDG_RUNTIME_DIR=/run/user/<user_id>
<user_id>は
$ id
で見ることができる

# rand
cargo add rand


# Xserver install
https://sourceforge.net/projects/vcxsrv/
ここからVcXsrvをインストール
するとXlaunchが入る. 色々設定し完了までする
cmdでipconfigを実行するとwslのIPv4アドレスが分かる
export DISPLAY=<WindowsのIPアドレス>:0.0
を実行
DISPLAY環境変数にWindowsマシンのIPアドレスを設定し、X Server経由でGUIアプリケーションを表示するための設定
wslを念のためアップデート
cmdで　wsl --update

# コンパイル、実行
cargo run

テスト
cargo test

ビルド
cargo build

# メモ

## フラグレジスタ
NV-BDIZC

N Negative flag : 演算結果の7bit目（最上位ビット）が１ならセット
V Overflow flag : プラス同士あるいはマイナス同士の演算で符号が変わったときにセット
- 常に１がセット
B Break command : BRK命令が行われたあとにセット
D Decimal mode  : これがセットされているとプロセッサは加算と減算の際にバイナリ算術に従う
I Interrupt disalbe : SEI命令の後にセット。セットされている間プロセッサはデバイスからの割り込みに応答しない
Z Zero flag     : 演算結果が0のときセット
C Carry flag    : 演算結果で最上位ビットがオーバーフローまたは最下位ビットがアンダーフローしたときにセット


## ページ
0x0000 ~ 0x00FF 255個 1ページ　特にこの0x0000~の１ページをゼロページという
このページは優遇されていて速くアクセスできる

## unwrap()
OptionやResultといった列挙型（enums）を扱う際に使用される便利なメソッドの一つ.

- Option<T>：これは、ある値が存在するかどうかを表現するために使用されます。値が存在する場合、Some(T)として表現され、値が存在しない場合、Noneとして表現されます。

- Result<T, E>：これは、関数が成功するか失敗するかを表現するために使用されます。成功する場合、Ok(T)として表現され、失敗する場合、Err(E)として表現されます。

unwrap()メソッドはこれらの列挙型から値を取り出すために使われます.
Optionに対してならSomeの値、Resultに対してならOkの場合の中の値を取り出す

ex)
let some_value: Option<i32> = Some(42);
let unwrapped = some_value.unwrap();

## event_pump(), poll_iter()
- イベントポンプは、イベントが発生する場所やソース(sdl_contextなど)からイベントを収集し、プログラムに提供するための抽象化です。通常、ウィンドウシステムからのユーザー入力（キーボードやマウスのイベントなど）や、その他の外部イベント（ファイルシステムの変更、ネットワークイベントなど）を受信し、プログラム内で処理可能な形式に変換します。

- poll_iter()は、イベントポンプから取得できるイベントを反復処理するためのメソッドです。このメソッドは、次のようなことを行います：
1 イベントポンプが管理するイベントキューから次のイベントを取得します。
2 取得したイベントをプログラム内で処理するために提供します。
3 イベントが処理された場合、キューから削除されます。

通常、このメソッドを無限ループ内で使用して、ユーザーからの入力やシステムイベントをリアルタイムに監視し、それに応じてアプリケーションの振る舞いを変更します。


## CPU Memory Map
[0x0000...0x1FFF] CPU RAM
- [0x0000...0x07FF]はWRAM, [0x0800...0x1FFF]はWRAMのミラーリング
- [0x0100...0x00FF]はスタック領域として使用する

[0x2000...0x3FFF] IO Port PPU 
- PPUレジスタ[0x2000...0x2007] これはアドレス空間[0x2008...0x3FFF]にミラーリングされる

[0x4000...0x401F] IO Port APU, Joypad

[0x4020...0x5FFF] Expansion Rom 
- Mapper. カートリッジの世代によって異なる方法で使用されるスペース

[0x6000...0x7FFF] Save RAM (拡張RAM)
- カートリッジに RAM スペースがある場合、カートリッジ上の RAM スペース予約される. ゲームの状態を保存したりなど.

[0x8000....0xFFFF] Prg ROM(カートリッジ上のプログラムROMスペース上にマップされる)
- [0x8000....0xBFFF]はprg romのLOW, [0xC000...0xFFFF]はprg romのHIGH
- NMI [0xFFFA,0xFFFB], RESET [0xFFFC,0xFFFD], IRQ/BRK [0xFFFE,0xFFFF]



## PPU

VRAMへのアクセス

アドレス ( 0x2006 ) とデータ ( 0x2007 )をエミュレートする場合
1.要求アドレスを Addr レジスタにロードする必要があります。2 バイトを 1 バイトのレジスタにロードするには、レジスタに 2 回書き込む必要があります

2.次に、CPU は PPU データ レジスタ (0x2007) からのデータ ロードを要求できます

3.最終的に PPU の内部バッファから値を取得するには、CPU は 0x2007 からもう一度読み取る必要があります

write[0x2006] = 0x60
write[0x2006] = 0x00
read[0x2007]  <- PPUは値をすぐ返せないので一回目はダミーを返す
PPUはcha_romにread[0x0600]する
chr_romからデータを受け取りストアする
インクリメント処理
CPUからread[0x2007]　（2回目）
PPUはストアした値を返す

## PPU Memory Map
PPUは6502からはマップされておらず、データの送受信はI/Oポートを通じて行う

[0x0000...0x0FFF] パターンテーブルLOW
[0x0100...0x1FFF] パターンテーブルHIGH

[0x2000...0x23BF] 画面1 ネームテーブル
[0x23C0...0x23FF] 画面1 属性テーブル
[0x2400...0x27BF] 画面2 ネームテーブル
[0x27C0...0x27FF] 画面2 属性テーブル

[0x2800...0x2BBF] 画面3 ネームテーブル
- [0x2000...0x23BF]のミラーリング

[0x28C0...0x2BFF] 画面3 属性テーブル
- [0x23C0...0x23FF]のミラーリング

[0x2C00...0x2FBF] 画面4 ネームテーブル
- [0x2400...0x27BF]のミラーリング

[0x2FC0...0x2FFF] 画面4 属性テーブル
- [0x27C0...0x27FF]のミラーリング

[0x3000...0x3EFF] 未使用
- [0x2000...0x2EFF]のミラーリング

[0x3F00...0x3F0F] BGパレットテーブル
[0x3F10...0x3F1F] スプライトパレットテーブル

[0x3F20...0x3FFF] 未使用
- [0x3F00...0x3F1F]のミラーリング

[0x4000...0xFFFF] 未使用
- [0x0000...0x3FFF]のミラーリング


## Frame
フレーム" はビデオ表示の単位を指します。一般的に、ビデオゲームのグラフィックスは瞬間ごとに画面に表示されます。この瞬間を "フレーム" と呼びます。
ファミコン（NES）において、1つのフレームは以下の要素から構成されます：

1 画面表示: 画面に表示されるビデオグラフィックス、すなわちゲームの背景、キャラクター、オブジェクト、テキストなどが含まれます。これらの要素はPPU（Picture Processing Unit）によって生成され、テレビ画面に表示されます。

2 ゲームロジック: ゲーム内のロジック、プレイヤーの入力の処理、キャラクターの動作、衝突判定などが1つのフレーム内で進行します。これにより、ゲームの進行状況が制御されます。

3 サウンドと音楽: ゲーム内のサウンドエフェクトや音楽も1つのフレーム内で生成され、テレビやモニターのスピーカーから出力されます。これにより、ゲームのオーディオ要素が提供されます。

ファミコン（NES）のゲームは、60フレーム/秒のフレームレートで表示されることが一般的でした。


## render
PPUから画面情報を取得し、それをframeに描画する

## APU

概要
4ch ノイズ
3ch 三角波
1,2ch 矩形波

矩形波
パラメータ：
- Duty比
- ボリューム
- ヘルツ