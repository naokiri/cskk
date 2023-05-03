# Changelog

All notable changes to this project will be documented in this file.

フォーマットは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に準じる。
バージョンの付けかたは [Semantic Versioning](https://semver.org/spec/v2.0.0.html) に準じる。

## [Unreleased]

### Changed

- モード遷移を修正し、無限に深いモード遷移をできないように修正。Registerの入れ子を10回までに制限する。
- Abortで変換確定前の状態に戻ってstateがおかしい状態になることを修正するため、変換確定時に自動的にデフォルトのモードに戻るように"Abort"
  の処理を変更。それに応じてルールファイルを修正。

### Fixed

- AZIKモードの重複したかな変換ルールを削除。
- default,AZIKモードの送り仮名時のC-gのルールを修正。

## [3.0.0] - 2023-02-05

### Added

- 補完機能の追加。補完に用いると指定した辞書の送りがな無しエントリを補完候補として出す。
- AZIKのrulesファイルにSKK向け独自拡張の「っ」の入力方法を追加

### Changed

- 補完機能のためAPIインタフェース変更。
- 一部モードの状態に確定済み文字列を返すように変更。
- Abortコマンドの挙動変更、単体で直前のモードに戻すように。
- rulesファイルのAbortを使っていた部分の修正。
- rulesファイルに補完モード関連のキーを追加。

### Fixed

- Tabが単体で押されていた時と他のキーと同時に押されていた時に違うキーとして扱われていた問題を修正。

## [2.0.0]

### Changed

- APIインタフェースのバグ修正による変更。

## [1.0.0]
