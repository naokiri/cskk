# Changelog

All notable changes to this project will be documented in this file.

フォーマットは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に準じる。
バージョンの付けかたは [Semantic Versioning](https://semver.org/spec/v2.0.0.html) に準じる。

## [Unreleased]

### Added

- 補完機能の追加。補完に用いると指定した辞書の送りがな無しエントリを補完候補として出す。

### Changed

- 補完機能のためAPIインタフェース変更。
- Abortコマンドの挙動変更、単体で直前のモードに戻すように。
- rulesファイルのAbortを使っていた部分の修正。
- rulesファイルに補完モード関連のキーを追加。

### Fixed

- Tabが単体で押されていた時と他のキーと同時に押されていた時に違うキーとして扱われていた問題を修正。

## [2.0.0]

### Changed

- APIインタフェースのバグ修正による変更。

## [1.0.0]
