Rust crate for moving around in MP4 containers. Does not and will not support any kind of media de/encoding.

Usage (not yet on crates.io):

`Cargo.toml`:
```toml
[dependencies]
mp4iter = {git = "https://github.com/jenslar/mp4iter.git"}
```

`src/main.rs`:
```rs
use mp4iter::Mp4;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;

    for atom_header in mp4.into_iter() {
        println!("{atom_header:?}")
    }

    // Derives duration for MP4 for longest track.
    println!("{:?}", mp4.duration());

    // Extracts track data for GoPro GPMF telemetry (handle name 'GoPro MET')
    let mut track = mp4.track("GoPro MET")?;
    println!("{track:#?}");

    // Iterate over raw sample data.
    for (i, result) in track.samples().enumerate() {
        let sample: Sample = result.unwrap(); // implements read + seek
        println!("{:04} {} bytes, duration: {}, timestamp: {}",
            i+1,
            sample.len(),
            sample.duration(),
            sample.relative()
        );
    }

    Ok(())
}
```