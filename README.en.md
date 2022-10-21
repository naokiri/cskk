![cskk logo](https://raw.githubusercontent.com/naokiri/cskk-icons/master/256x256/apps/cskk.png)

# LibCSKK

Cobalt SKK library.

CSKK is a library to implement Simple Kana-Kanji henkan.

Of course, this library is named as 'CSKK' because it is extensionally equal to SKK.

For Fcitx5: [fcitx5-cskk](https://github.com/fcitx/fcitx5-cskk)

Logos and icons: [cskk-icons](https://github.com/naokiri/cskk-icons)

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

When you have root priviledge, run the following.
`cargo cbuild --release` 
This installs the files generated under target/{arch}/release to proper system directories and the data under assets/ to proper proper data directory's libcskk/ direcotry.

```shell
    cargo cinstall --release
```

To install to non-standard directories, append following options like this. See
[cargo-c](https://github.com/lu-zero/cargo-c)  for details.

```shell
    cargo cinstall --release --prefix="/tmp" --datadir="$HOME/.local/share"
```

- prefix: Prefix appended to the default libdir, includedir, pkgconfigdir, and datarootdir. Default is '/usr/local'
- libdir: Directory to install the library. Default is '/lib'
- includedir: Directory to install the header file. Default is '/include'
- pkgconfigdir: Directory to install the .pc file for pkg-config. Default is '/lib/pkgconfig'
- datarootdir: Directory to install the data files (assets directory in cskk project's case). Default is 'share'
- datadir: Override datarootdir. An option to install data to a directory out of prefixed dirs. Default is unset (use datarootdir as is)

## Development status

### Simulating DDSKK feature

- [x] ひらがな入力
- [x] カタカナ入力・カタカナモード
- [x] ｶﾀｶﾅ入力・ｶﾀｶﾅモード
- [x] Basic 漢字変換
- [x] static dictionary
- [x] user dictionary
  - ddskk compatible since v0.11.0
- [ ] 接頭辞・接尾辞変換
- [x] 数値変換
- [x] auto-start-henkan   
- 実装見込が現在ないもの
    - [ ] Kuten 変換
    - [ ] 今日の日付入力
    - [ ] 異字体変換
    - [ ] SKK辞書サーバー対応
    - [ ] 外部辞書

## Simulating ueno/libskk feature

- [x] 句読点設定
- [x] AZIK rule
- [ ] Nicola rule

### C FFI + IME plugin

- [x] C ABI library for fcitx5-skk
  Reference https://github.com/naokiri/fcitx5-cskk

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

