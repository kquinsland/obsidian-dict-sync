# cross platform build notes

Figure out what options I have

```shell
❯ rustup target list | grep windows
aarch64-pc-windows-msvc
i586-pc-windows-msvc
i686-pc-windows-gnu
i686-pc-windows-msvc
x86_64-pc-windows-gnu
x86_64-pc-windows-msvc
```

Not sure what additional baggage `msvc` will bring, will stick with `gnu` variants now.
Also not interested in 32 bit support. I don't have any arm based windows hosts that I can use for testing so that'll be omitted, too.

```shell
❯ rustup target add x86_64-pc-windows-gnu
info: downloading component 'rust-std' for 'x86_64-pc-windows-gnu'
info: installing component 'rust-std' for 'x86_64-pc-windows-gnu'
```

And then try building:

```shell
❯ cargo build --target x86_64-pc-windows-gnu
<...>
error: linker `x86_64-w64-mingw32-gcc` not found
```

Or maybe I just use [`cross`](https://github.com/cross-rs/cross) which keeps things nice and tidy inside Docker?

```shell
❯ cargo install cross
<...>
❯ cross --version
cross 0.2.5
[cross] note: Falling back to `cargo` on the host.
cargo 1.76.0 (c84b36747 2024-01-18)
```

And giving it a go:

```shell
❯ cross build --target x86_64-pc-windows-gnu
Unable to find image 'ghcr.io/cross-rs/x86_64-pc-windows-gnu:0.2.5' locally
0.2.5: Pulling from cross-rs/x86_64-pc-windows-gnu
<...>
```
