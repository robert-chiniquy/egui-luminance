# dev server

```
$ yarn serve
```

http://localhost:8080

# Rust compile error due to glsl! macro

If you get this error: `error[E0554]: `#![feature]` may not be used on the stable release channel`
Then run: `rustup override set nightly` (you'll need to already have the nightly toolchain installed)