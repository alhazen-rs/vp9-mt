# vp9-mt

A multi-threaded, pure-Rust VP9 decoder: a fork of [rusty_vp9](https://crates.io/crates/rusty_vp9)
0.1.1 that decodes tile columns and loop-filters planes on worker threads, in safe Rust (checked
with Miri). Output is bit-identical to the single-threaded decoder (and to libvpx) for every
thread count.

```rust
let mut dec = vp9_mt::Vp9Decoder::with_threads(0); // 0 = available parallelism
dec.push(&packet, Some(pts))?;
while let Ok(picture) = dec.next_picture() {
    let (y, stride) = picture.plane(0); // u16 samples; picture.plane_size(0) are visible
}
```

On a 3836x2072 10-bit stream with 8 tile columns, 12 threads decode 2.6× faster than one
(45 fps vs 17.5 fps).
See [CHANGES.md](CHANGES.md) for what differs from rusty_vp9, and [NOTICE](NOTICE).

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
