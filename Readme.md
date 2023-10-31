# install
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
## 参考
- NesDevWiki
- writing Nes emulator in Rust
- Nes on FPGA
- 6502 reference
- Nes研究室
- ファミコンエミュレータの創り方
- Writing NES Emulator in Rustをやった

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



## PPU（Picture Processing Unit）

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
実際の描画処理を行うモジュール

## 画面描画
- タイル
タイルは、画面の小さな正方形の描画ユニットです。通常、8x8ピクセルまたは8x16ピクセルの正方形です。
NESの画面は、複数のタイルが配置されてできており、これらのタイルを組み合わせて画面全体を描画します。
タイルは背景やスプライト（キャラクター）など、画面上のさまざまな要素の基本単位として使用されます。

- タイルマップ
タイルマップは、画面上のどの位置にどのタイルが配置されるかを示すデータ構造です。
タイルマップは、座標系に基づいて画面上の各位置に対応するタイル番号を持ち、これによって画面がどのように描かれるかを制御します。

- パターンテーブル
NESのPPUは、背景タイルとスプライトタイルのためにパターンテーブルと呼ばれるメモリ領域にタイルのグラフィックスデータを格納します。
パターンテーブルには、各タイルのビットマップが保存されており、これを読み取って実際の描画に使用します。

- ネームテーブル
ネームテーブルは、背景のタイル（タイルマップ）の配置情報を格納するメモリ領域です。
通常、NESのネームテーブルは2つあり、それぞれが画面の左半分と右半分（または上半分と下半分）に対応します。これは、水平ミラーリングまたは垂直ミラーリングのために使用されます。
各ネームテーブルは、画面上の各位置に対応するタイルの番号を持っており、これによって背景の描画情報が管理されます。
ゲームのプログラマーとデザイナーは、ネームテーブルを編集して、背景の配置を設定します。

- 属性テーブル
属性テーブルは、ネームテーブル内のタイルに対応する背景の色情報を管理します。
NESの画面は、背景が8x8ピクセルのタイルで構成され、各タイルには16色のパレットから選択した色が割り当てられます。属性テーブルは、どのパレットがどのタイルに適用されるかを示します。
通常、1つの属性テーブルは4つのネームテーブルに対応し、ネームテーブル内の16x16タイルグリッドごとに属性情報を格納します。

エミュレータは、ゲームカートリッジ内のデータからネームテーブルと属性テーブルを読み取り、これらの情報をもとに画面の背景描画を再現します。

- スキャンライン
ビデオディスプレイやカメラの画像処理に関連する用語で、特に古典的なCRT（陰極線管）モニターやテレビにおいて重要な概念です。スキャンラインは、画面を水平に走査する際に一行分の画素（ピクセル）情報を表します。
画面の表示は通常、水平に一行ずつ走査されます。これにより、画面全体が描かれます。各水平行が「スキャンライン」と呼ばれ、画面全体は複数のスキャンラインから構成されます。

- ミラーリング
古典的なゲームカートリッジのデータ配置に関連する概念で、主に古いゲーム機のカートリッジ内のメモリ構造を指します。
水平ミラーリングでは、PPUがメモリの一部を複製し、同じ情報を複数の場所に反映させます。
この方式では、画面の上半分と下半分の情報が同じであるため、メモリ上で同じデータを2つの場所に保存します。
垂直ミラーリングでは、PPUがメモリの一部を反映させ、水平方向では同じ情報が表示されます。
この方式では、画面の左半分と右半分の情報が同じであるため、メモリ上で同じデータを2つの場所に保存します。

ゲームカートリッジ内のメモリチップは、その容量に応じてコストがかかります。ゲーム内のデータ（ゲームプログラムやゲームデータ）は、このメモリチップに格納されます。しかし、ゲーム内のデータの中には同じ情報を複数回使用することがあるため、同じデータを複数の場所に配置しそのデータを異なるアドレスからアクセスすることで画面に同じデータを表示できます。つまり、同じデータを複数の場所に配置することなく、1つのデータセットを再利用することができるのです。


## マッパー
カートリッジ側に仕組まれているしくみ
ファミコンは
プログラムのサイズ(prg_rom)は最大32kB(0x8000~0xFFFF)
キャラは8kB(PPUのアドレスでいうと0x0000~0x1FFF)

しかし例えばDQ3は256kB, DQ4は512kB
どうやって動かすかというと本体からカセットに指令を出す。指令を出したら読む場所をずらすようにして
ずらしたところを返すように要求する
例えば256kBの場合、16kBを一つのまとまり(bank)としてもつ。この場合bankは16個に分けられる
一気に見れるのは最大32kBより2bankまで。bank0, bank1やbank3,bank7など。
この見ているbankを切り替えることができるのがマッパー
マッパーが違うというのは、指令の出し方（マッパーの切り替え方）やbankの大きさ、chr_romも同じように入れ替えをしたいなどといった仕様が違うということ


## APU

概要
1,2ch 矩形波
3ch 三角波
4ch ノイズ

矩形波
パラメータ：
- Duty比
- ボリューム
- ヘルツ

エンベロープ
矩形波とノイズで利用されボリュームをのこぎり型のように波打つようにするもの

1ch 矩形波
$4000 音色・音量
DDLERRRR
R:レート
  - エンベロープ有効ならエンベロープ速度の指定、無効なら音量の指定（max15）
E:エンベロープフラグ
  - 0なら有効、1なら無効
L:キーオフカウンタフラグ・エンベロープループフラグ
  - 0でキーオフカウンタ有効、エンベロープループ無効
  - 1でキーオフカウンタ無効、エンベロープループ有効
D:Duty比
  - 00:12.5%  01:25%  10:50%  11:75%

$4001 スイープ
FSSSHRRR
R:周波数変更量
H:周波数変更方向
S:周波数変更タイマカウント数
F:スイープ有効フラグ

$4002 周波数下位
FFFFFFFF
F:周波数

$4003 周波数上位・キーオンオフ
CCCCCFFF
F:周波数
C:キーオンカウント値

2ch 矩形波
$4004 音色・音量
DDLERRRR
R:レート
  - エンベロープ有効ならエンベロープ速度の指定、無効なら音量の指定（max15）
E:エンベロープフラグ
  - 0なら有効、1なら無効
L:キーオフカウンタフラグ・エンベロープループフラグ
  - 0でキーオフカウンタ有効、エンベロープループ無効
  - 1でキーオフカウンタ無効、エンベロープループ有効
D:Duty比
  - 00:12.5%  01:25%  10:50%  11:75%

$4005 スイープ
FSSSHRRR
R:周波数変更量
H:周波数変更方向
S:周波数変更タイマカウント数
F:スイープ有効フラグ

$4006 周波数下位
FFFFFFFF
F:周波数

$4007 周波数上位・キーオンオフ
CCCCCFFF
F:周波数
C:キーオンカウント値


3ch 三角波
$4008 音長
FLLLLLLL
L:音長
F:音長・キーオフカウンタ有効フラグ
  - 0で有効、1で鳴りっぱなし

$4009 未使用

$400A 周波数下位
FFFFFFFF
F:周波数

$400B 周波数上位・キーオンオフ
CCCCCFFF
F:周波数
C:キーオンカウント値


ノイズ
$400C　音量
XXLERRRR
X:未使用
L:キーオフカウンタフラグ・エンベロープループフラグ
  - 0でキーオフカウンタ有効、エンベロープループ無効
  - 1でキーオフカウンタ無効、エンベロープループ有効
R:レート

$400D　
未使用

$400E　周波数
RXXXFFFF
F:ノイズ周波数
X:未使用
R:長周期、短周期
  - 0で長周期

$400F キーオン・オフ
CCCCCXXX
X:未使用
C:キーオフカウント値
