## 開発者向け

開発中の確認は主に`cargo build` と `cargo test`でできるようにしています。

Nightlyでのチェックはrustupでnightlyをインストール後、
```shell
    cargo clean && cargo +nightly check
```

C ABI ライブラリを確認する場合、以下のような手作業です。
```shell
    cargo cbuild 
    cp target/x86_64-unknown-linux-gnu/debug/libcskk.h ./c_tests/
    $(CC) c_tests/c_shared_lib_test.c -L ./target/x86_64-unknown-linux-gnu/debug/ -lcskk -o c_tests/lib_test
    LD_LIBRARY_PATH=./target/x86_64-unknown-linux-gnu/debug ./c_tests/lib_test
```

## Notes for Developers

`cargo build` and `cargo test` shall be enough for most of the development.

To check in nightly channel, install nightly on rustup and then run 
```shell
    cargo clean && cargo +nightly check
```

To generate the C ABI library,
```shell
    cargo cbuild 
    cp target/x86_64-unknown-linux-gnu/debug/libcskk.h ./c_tests/
    $(CC) c_tests/c_shared_lib_test.c -L ./target/x86_64-unknown-linux-gnu/debug/ -lcskk -o c_tests/lib_test
    LD_LIBRARY_PATH=./target/x86_64-unknown-linux-gnu/debug ./c_tests/lib_test
```