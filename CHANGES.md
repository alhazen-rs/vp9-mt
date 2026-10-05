# Changes from rusty_vp9 0.1.1

vp9-mt is a fork of [rusty_vp9](https://crates.io/crates/rusty_vp9) 0.1.1 (Apache-2.0, by Mata
Network). Changes, all in the decoder:

- **Tile-column threading** (`src/decode.rs`, safe Rust): a VP9 tile column depends only on
  itself, so each column (all its tile rows, in order) is decoded by its own worker
  `Reconstructor` with column-local planes, mode-info grid, segment map and contexts; read-only
  state (probabilities, references, previous-frame grids) is shared by `Arc`/clone. Results are
  copied into the frame afterwards, columns in parallel over disjoint `split_at_mut` pieces
  (`run_parallel`, `scatter_columns`); worker buffers are reused across frames (`ColumnBufs`).
  Pixel/grid indexing goes through the column origin (`Plane::x0`, `mi_at`).
- **Parallel loop filter per plane** (`src/loopfilter.rs`, `loop_filter_frame_parallel`): Y, U
  and V are filtered on separate threads, each in the original raster order.
- (An earlier version shared the frame buffers between workers through bitwise copies and ran a
  superblock-row wavefront loop filter. Miri showed that to be undefined behaviour — overlapping
  `&mut` slices of whole planes on several threads — and it was replaced by the design above.)
- **`Vp9Decoder::with_threads(n)`** (`src/lib.rs`): `0` uses the available parallelism, `1` is the
  original single-threaded path. `Vp9Decoder::new()` uses the available parallelism.
- **`Vp9Decoder::next_picture()` / `Picture`** (`src/lib.rs`): returns the decoder's reference
  buffer without the cropped byte copy that `next_frame()` makes (that copy was ~15% of decode
  time at 4K). `next_frame()` is unchanged.
- Two unused `mut`s removed in `src/transform.rs` tests (compiler warnings).
- **Removed the encoder** (`src/encode/` and its re-exports): vp9-mt is decoder-only.
- Tests: `tests/threads.rs` (threaded output equals single-threaded, 100× stress at 16 threads,
  and a 512×64 two-tile clip small enough to run under Miri:
  `MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test -p vp9-mt --test threads tiny`).

Every thread count produces bit-identical output; the libvpx conformance vectors are checked at
1, 2, 4, 8 and 16 threads by the gpui-native-video-player workspace
(`crates/video-core/tests/vp9_conformance.rs`).
