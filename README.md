# vp9-mt

[![crates.io](https://img.shields.io/crates/v/vp9-mt.svg)](https://crates.io/crates/vp9-mt)
[![docs.rs](https://img.shields.io/docsrs/vp9-mt)](https://docs.rs/vp9-mt)
[![CI](https://github.com/alhazen-rs/vp9-mt/actions/workflows/ci.yml/badge.svg)](https://github.com/alhazen-rs/vp9-mt/actions/workflows/ci.yml)

A multi-threaded, pure-Rust VP9 decoder, in safe Rust.

vp9-mt is a fork of [rusty_vp9](https://crates.io/crates/rusty_vp9) 0.1.1 that decodes tile
columns and loop-filters planes on worker threads. Its output is **bit-identical** to the
single-threaded decoder, and to libvpx, at every thread count.

It's the VP9 decoder of [Alhazen](https://github.com/alhazen-rs), a native media engine for
Rust, and works on its own too.

## Highlights

- **Fast:** on a 3836×2072 10-bit stream with 8 tile columns, 12 threads decode 2.6× faster
  than one (45 fps vs 17.5 fps).
- **Correct:** checked against libvpx's MD5s for the official VP9 test vectors at 1, 2, 4, 8
  and 16 threads, plus a 100-run threaded stress test.
- **Safe:** no `unsafe` in the threading code; Miri (Tree Borrows and Stacked Borrows) runs
  the threaded decoder clean.
- **No C, no build scripts, no system dependencies.**
- 8- and 10-bit streams.

## Usage

```toml
[dependencies]
vp9-mt = "0.1"
```

```rust
use vp9_mt::Vp9Decoder;

let mut dec = Vp9Decoder::with_threads(0); // 0 = use all available cores; 1 = single-threaded

for packet in packets {
    // One VP9 frame (or superframe) per call, as stored in WebM/MP4/IVF.
    dec.push(&packet.data, Some(packet.pts))?;
    while let Ok(picture) = dec.next_picture() {
        let (w, h) = (picture.width(), picture.height());
        let bits = picture.bit_depth();                  // 8 or 10
        let (y, y_stride) = picture.plane(0);            // samples are u16 at any bit depth
        let (u, u_stride) = picture.plane(1);
        let (v, v_stride) = picture.plane(2);
        let (y_w, y_h) = picture.plane_size(0);          // visible size of the plane
        let full_range = picture.full_range();
        // ... convert or display
    }
}
dec.flush(); // after a seek: drop reference frames and anything pending
```

| `Vp9Decoder` | |
|---|---|
| `new()` | Single-threaded decoder. |
| `with_threads(n)` | `n` worker threads for tile columns (`0` = available parallelism). Threading only helps streams encoded with several tile columns (most HD/4K encodes). |
| `push(data, pts)` | Feeds one compressed frame or superframe. |
| `next_picture()` | The next decoded picture, with planes as `u16` samples. |
| `next_frame()` | The next frame as a `DecodedFrame` (rusty_vp9's original API). |
| `pending()` | Frames decoded but not yet taken. |
| `flush()` | Resets the decoder (seek). |

Lower-level helpers from rusty_vp9 are public too: `parse_uncompressed_header`,
`consume_compressed_header`, `FrameHeader`.

## How the threading works

A VP9 tile column depends only on itself, so each column (all its tile rows, in order) is
decoded by its own worker, with column-local planes, mode-info grid, segment map and contexts.
Read-only state (probabilities, reference frames, the previous frame's grids) is shared. The
columns are then copied into the frame in parallel through disjoint `split_at_mut` slices,
and worker buffers are reused across frames. The loop filter then runs Y, U and V on separate
threads, each in the original raster order.

All of this is safe Rust. [CHANGES.md](CHANGES.md) lists every change from rusty_vp9.

## Testing

```bash
cargo test --release                                          # threaded output == single-threaded, bit for bit
cargo +nightly miri test --test threads tiny                  # Miri on the tiny 2-tile clip
```

The libvpx conformance suite (the official VP9 test vectors at 1–16 threads) runs in
[alhazen-core](https://github.com/alhazen-rs/alhazen-core), which downloads the vectors.

## License

Apache License, Version 2.0 ([LICENSE](LICENSE)). vp9-mt is a modified version of rusty_vp9 by
Mata Network, also Apache-2.0; see [NOTICE](NOTICE) and [CHANGES.md](CHANGES.md).
