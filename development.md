# Development note

## deploy process
Not automated yet.
Cargo.tomlおよび.github/workflows/continuous_testing.yaml内の確認するバージョンを書き換え
githubでReleaseからDraft a new releaseしてvx.y.zのようなタグを作ってreleaseブランチからリリースする
github workflowでartifactがアップロードされる。

その後cargoにもリリースするため

    git checkout vx.y.z
    cargo login && cargo publish --dry-run 
    # 結果を見てよさそうなら 
    cargo publish

という手順を踏む。

## ローカルで試用する
cskkで何かを変えてIMEで確認したい時、静的リンクでない場合ビルド時と実行時両方に指定する。

fcitx5を例にすると
    
    # cskkを別ディレクトリにインストール
    cargo cinstall --prefix=/tmp/cskkdir
    # fctix5-cskkでビルド時
    PKG_CONFIG_PATH=/tmp/cskkdir/lib/pkgconfig cmake -B ./build && cd build && make
    # fcitx5 の実行時
    LD_LIBRARY_PATH=/tmp/cskkdir/lib/cskk FCITX_ADDON_DIRS=/home/naoaki/src/fcitx5-cskk/build/src:/usr/lib/x86_64-linux-gnu/fcitx5 fcitx5 --verbose=*=5 

このようにしてローカルビルドのlibcskkとfcitx5-cskkを試用できる。assets

#### cskk
cskk v0.4では徐々に機能追加をしてしまったせいでprocess_key_event_innerのコメントの通り

1. rom2kana可能? -yes-> かな入力として処理 (大文字のみcompositionmode変更コマンドとしても処理)
2. 現在のCompositionMode内で解釈されるコマンド？ -yes-> compositionmode用コマンドとして処理
3. (delegate するタイプのinstructionだった場合) ループ
4. rom2kana継続可能 or ascii？ -yes-> 継続入力として処理
5. rom2kana継続不可能 -all-> Flush後に入力として処理

と素の入力部分がとても散らかっている。
libskk同様に先にコマンドとして解釈しようとしたい。
inputmode+compositionmode -> Instruction 配列というのが現在の状態だが設定ファイルに書きだす？
rom2kanaを一本化するためにprocess_key_event_innerを書き直す。

まずはv0.4.0のInstructionで設定ファイルにそのままおこせないものをどうするか
##### コマンド類
InputMode/CompositionMode以外の条件分岐があるものを変更する

Direct
- 大文字が来た時にモード変更もInstructionで行い、キー処理を終わらないようなInstructionにしている。libskkでもNoneStateHanderで特殊処理していた。これをコマンドから外す。

Precomposition
- Q以外の大文字が来た時に送り仮名モードへ変更を行い、キー処理を終わらないようなInstructionにしている。libskkではStartStateHandler内でコマンドでなくis_upperで特殊処理していた。これもコマンドから外す。
- delegateが存在する。これもnext_candidateとして一括処理する？

Composition
- delegated時のみ候補が空ならRegisterモードへ再度送っている。
- candidate_listが正しい状態かをチェックして、candidtate_listを更新するか、次の候補へ移動するか、Registerモードへ送るかを決めている。これをnext_candidate、previous_candidateとしてハンドラ内では判断しないようにして、コマンド処理側でcandidate_listの何番目を指しているかによって動作を変える。

### 
まず上記next/previous candidateの変更だけでリファクタリング後、Instructionの不要部分を消してからマージしたい。

## bug or feature?
### Shift 押しっぱなしの送り仮名
cskk current: S i N I -> ▼死に
libskk: S i N I -> ▼死んい

### abbrevモード不安定？
/を押してもabbrevにならない? 
/ l e space C-g space でステートが残る？
### その他
?が全角?ではなく半角になる?