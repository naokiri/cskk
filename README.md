# LibCSKK
[![Build Status](https://travis-ci.com/naokiri/cskk.svg?branch=master)](https://travis-ci.com/naokiri/cskk)
[![Build Status](https://github.com/naokiri/cskk/workflows/Test/badge.svg)](https://github.com/naokiri/cskk/actions)

Cobalt SKK library. 

CSKK is a library to implement Simple Kana-Kanji henkan.

Of course, this library is named as 'CSKK' because it is extensionally equal to SKK.

[ddskk]: http://openlab.ring.gr.jp/skk/ddskk.html


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

## Develop
There's no convenient makefile-like scripts here. `cargo build` and `cargo test` shall be enough for most of the development. 

To run tests on .so file,
```shell
    cargo build
    cbindgen --config cbindgen.toml --crate cskk --output tests/libcskk.h
    $(CC) tests/c_shared_lib_test.c -L ./target/debug/ -lcskk -o tests/lib_test
    LD_LIBRARY_PATH=./target/debug ./tests/lib_test
```

## Development roadmap
### SKK in rust, simulating the keyevent.
First goal is to simulate skk like libskk and be able to run something similar that libskk does for library testing.
 
e.g. 
- "N e o C h i SPC N" -> "▼ねお*ち【▽n】"

### C FFI + IME plugin
Second goal.

### Nicola support?
maybe in this lib, maybe better in other lib.

## Copyright
Copyright (C) 2018  Naoaki Iwakiri

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

