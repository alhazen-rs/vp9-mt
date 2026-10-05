# Changes from rusty_vp9 0.1.1

vp9-mt is a fork of [rusty_vp9](https://crates.io/crates/rusty_vp9) 0.1.1 (Apache-2.0, by Mata
Network). Changes, all in the decoder:

- **Tile-column threading** (`src/decode.rs`): within a tile row, tile columns are decoded by
  worker threads (`TileWorker`, `run_workers`). Each worker shares the frame buffers and owns its
  symbol counts, left contexts and coefficient scratch; counts are merged after each tile row.
- **Loop-filter wavefront** (`src/loopfilter.rs`): superblock rows are filtered in parallel; row
  `r` processes column `c` once row `r-1` has finished column `c+1`, which reproduces the
  raster-order result exactly.
- **`Vp9Decoder::with_threads(n)`** (`src/lib.rs`): `0` uses the available parallelism, `1` is the
  original single-threaded path. `Vp9Decoder::new()` uses the available parallelism.
- **`Vp9Decoder::next_picture()` / `Picture`** (`src/lib.rs`): returns the decoder's reference
  buffer without the cropped byte copy that `next_frame()` makes (that copy was ~15% of decode
  time at 4K). `next_frame()` is unchanged.
- **Removed the encoder** (`src/encode/` and its re-exports): vp9-mt is decoder-only.
- Tests: `tests/threads.rs` (threaded output equals single-threaded, 100× stress at 16 threads).

Every thread count produces bit-identical output; the libvpx conformance vectors are checked at
1, 2, 4, 8 and 16 threads by the gpui-native-video-player workspace
(`crates/video-core/tests/vp9_conformance.rs`).
