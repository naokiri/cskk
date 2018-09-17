# LibCSKK
[![Build Status](https://travis-ci.org/naokiri/cskk.svg?branch=master)](https://travis-ci.org/naokiri/cskk)
Cobalt SKK library. 

CSKK is a library to implement Simple Kana-Kanji henkan.

Of course, this library is named as 'CSKK' because it is extensionally equal to SKK.

[ddskk]: http://openlab.ring.gr.jp/skk/ddskk.html


# Development roadmap
## SKK in rust, simulating the keyevent.
First goal is to simulate skk like libskk and be able to run something similar that libskk does for library testing.
 
e.g. 
- "N e o C h i SPC N" -> "▼ねお*ち【▽n】"

## C FFI + IME plugin
Second goal.

## Nicola support?
maybe in this lib, maybe better in other lib.
