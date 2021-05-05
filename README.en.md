

# LibCSKK

Cobalt SKK library.

CSKK is a library to implement Simple Kana-Kanji henkan.

Of course, this library is named as 'CSKK' because it is extensionally equal to SKK.

Reference
- ddskk: http://openlab.ring.gr.jp/skk/ddskk.html
- libskk: https://github.com/ueno/libskk

## Build requirement

- libxkbcommon

In Ubuntu e.g.

```shell
    sudo apt install libxkbcommon-dev
```

- cbindgen

```shell
    cargo install --force cbindgen
```

- cargo-c

```shell
    cargo install --force cargo-c
```

## Install

Run

```shell
    cargo cinstall --release
    mkdir -p ~/.local/share/libcskk
    cp -r ./shared/* ~/.local/share/libcskk
```

To install to non-standard directories, append following options like this. See
cargo-c (https://github.com/lu-zero/cargo-c) for details.

```shell
    cargo cinstall --release --prefix=/usr --includedir=/tmp/other/place
```

- prefix: Prefix appended to the default libdir, includedir, and pkgconfigdir. Default is '/usr/local'
- libdir: Directory to install the library. Default is '/lib'
- includedir: Directory to install the header file. Default is '/include'
- pkgconfigdir: Direcotry to install the .pc file for pkg-config. Default is '/lib/pkgconfig'

## Develop

`cargo build` and `cargo test` shall be enough for most of the development.

To generate the C ABI library,

```shell
    cargo cbuild 
    cp target/x86_64-unknown-linux-gnu/debug/libcskk.h ./tests/
    $(CC) tests/c_shared_lib_test.c -L ./target/x86_64-unknown-linux-gnu/debug/ -lcskk -o tests/lib_test
    LD_LIBRARY_PATH=./target/x86_64-unknown-linux-gnu/debug ./tests/lib_test
```

## Development status

### Simulating DDSKK feature

- [x] ひらがな入力
- [x] カタカナ入力・カタカナモード
- [x] ｶﾀｶﾅ入力・ｶﾀｶﾅモード
- [x] Basic 漢字変換
- [x] static dictionary
- [x] user dictionary
    - not ddskk compatible
- [ ] 接頭辞・接尾辞変換
- [ ] 数値変換
- [ ] auto-start-henkan
    - ミスにより現在,.のみ
- 実装見込が現在ないもの
    - [ ] Kuten 変換
    - [ ] 今日の日付入力
    - [ ] 異字体変換
    - [ ] SKK辞書サーバー対応
    - [ ] 外部辞書

## Simulating ueno/libskk feature

- [ ] 句読点設定
- [ ] AZIK rule
- [ ] Nicola rule

### C FFI + IME plugin

- [ ] C ABI library for fcitx5-skk
  https://github.com/naokiri/fcitx5-skk 参照。

### Better development env, publish env

- [ ] github projects board や issue にこれらのリストを移す
- [ ] changelog

## Copyright

Copyright (C) 2018 Naoaki Iwakiri

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not,
see <https://www.gnu.org/licenses/>.

