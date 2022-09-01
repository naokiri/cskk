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

## libskk

libskkを真似てazik等設定可能にしたい

### 設定ファイル
metadata
rom-kana/
keymap/
metadataを探してよみ、rom-kana,keymapを読めるようにしている。
ファイルを分けているのは読む時の利便性か。実際には干渉しないようにディレクトリごとに設定自体は不可分。
metadata分けておくのは採用し、cskkではrom-kanaもkeymap(command)も同一ファイルとしてしまう。



rom-kana
rom-kana converterがファイルを読むようにして終わり

command
#### libskk
1. Stateごとにcommandの文字列を探して、存在し、かつ現在Stateが処理できればcommandとして処理
    1.1. ただし、コマンドによってrom_kana_converterが処理できない場合のみ処理等アドホックに無視されたりする。
2. commandとして処理できなかったらstateに応じたinputmodeの素の入力として処理できれば処理 (Stateによる)
3. stateが変更されており、キー処理が終了していなければ再度キー入力を新しいstateのハンドラで処理するため、ループする。実際のコードではStartStateから'next-candidate'の時のみSelectStateハンドラへ処理が移譲されている。 cskkと同様で、PreComposition->CompositionSelection->Registerの二段飛ばしを実装するため？

#### handler
- NoneStateHandler

preeditが存在しうる(=かなconvertがある)入力時全般。かな入力や変換登録のための入力。
■モード(Directモード)にあたる？

- KutenStatehandler

\コマンドのkuten変換時。コードポイントからなんか出している。詳細不明。

- AbbrevStateHandler

/コマンドのAbbrev変換State。ラテン文字の見出しから変換するモード。

- StartStateHandler

▽モード(PrecompositionとPrecompositionOkurigana)にあたる？
libskkでは送り仮名の有無も含めて同一ハンドラ。

- SelectStateHandler

▼モード

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