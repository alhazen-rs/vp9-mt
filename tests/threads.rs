//! vp9-mt's threaded paths (tile columns, loop-filter wavefront) must reproduce the
//! single-threaded output bit for bit. `tests/data/tiles4.ivf` is 1024x576 with 4 tile columns.

fn ivf_frames(bytes: &[u8]) -> Vec<&[u8]> {
    assert_eq!(&bytes[..4], b"DKIF");
    let header_len = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    let mut off = header_len;
    let mut out = vec![];
    while off + 12 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        off += 12;
        out.push(&bytes[off..off + len]);
        off += len;
    }
    out
}

fn decode(frames: &[&[u8]], threads: usize) -> Vec<Vec<Vec<u8>>> {
    let mut dec = vp9_mt::Vp9Decoder::with_threads(threads);
    let mut out = vec![];
    for f in frames {
        dec.push(f, None).unwrap();
        while let Ok(p) = dec.next_frame() {
            out.push(p.planes);
        }
    }
    dec.flush();
    while let Ok(p) = dec.next_frame() {
        out.push(p.planes);
    }
    out
}

const TILES4: &[u8] = include_bytes!("data/tiles4.ivf");

#[test]
fn threaded_output_equals_single_threaded() {
    let frames = ivf_frames(TILES4);
    let reference = decode(&frames, 1);
    assert_eq!(reference.len(), frames.len());
    for threads in [0, 2, 3, 4, 8, 16] {
        assert!(decode(&frames, threads) == reference, "threads={threads} differs from threads=1");
    }
}

#[test]
fn threaded_decode_is_stable_under_repetition() {
    let frames = ivf_frames(TILES4);
    let reference = decode(&frames, 1);
    // 100 runs under --release (CI); 10 in debug builds, where each run is ~40x slower.
    let runs = if cfg!(debug_assertions) { 10 } else { 100 };
    for run in 0..runs {
        assert!(decode(&frames, 16) == reference, "run {run} differs");
    }
}

#[test]
fn picture_matches_cropped_frame() {
    let frames = ivf_frames(TILES4);
    let mut dec = vp9_mt::Vp9Decoder::with_threads(4);
    dec.push(frames[0], Some(7)).unwrap();
    let pic = dec.next_picture().unwrap();
    assert_eq!((pic.width(), pic.height(), pic.bit_depth(), pic.subsampling()), (1024, 576, 8, (1, 1)));
    assert_eq!(pic.pts, Some(7));
    let frame = pic.to_decoded_frame();
    for p in 0..3 {
        let (data, stride) = pic.plane(p);
        let (w, h) = pic.plane_size(p);
        let cropped: Vec<u8> = (0..h).flat_map(|y| data[y * stride..y * stride + w].iter().map(|&s| s as u8)).collect();
        assert_eq!(cropped, frame.planes[p]);
    }
}
