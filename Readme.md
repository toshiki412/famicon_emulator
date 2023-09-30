cargoの使い方

ビルド
cargo build

実行
cargo run

テスト
cargo test

utf-8にしないとエラー



6502(8bit CPU)のステータスレジスタ
CZIDB-VN

C　キャリーフラグ:キャリー発生およびボローが発生しなかったときセット
Z　ゼロフラグ:演算結果が0のときセット。さらにロードのみでもその値によってセット
I　インタラプトフラグ:セットすると割り込みが禁止
D　デシマルフラグ:セットするとBCDモードで動作
B　ブレークフラグ:BRK割り込みを発生
-　未使用:常に1
V　オーバーフローフラグ:演算結果が符号付き8ビットを超えたときセット
N　ネガティブフラグ:演算結果が負のときセット。ロードのみでもその値によってセット



NES CPUはリトルエンディアンを使用
例えば
LDA $8000  <=> ad 00 80  //ad 80 00ではない


rustでは関数の戻り値を書く場合、;を付けない
;を付けると文として認識されてしまう。

メモリアドレッシングモード
