Experimental Rust crate for iterating MP4 containers, i.e. things may break. Does not and will not support any kind of media de/encoding.

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
    let mut mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;
    
    for atom_header in mp4.into_iter() {
        println!("{atom_header:?}")
    }

    // Derives duration for longest track.
    println!("{:?}", mp4.duration());

    // Extracts offsets for GoPro GPMF telemetry (handler name 'GoPro MET')
    println!("{:#?}", mp4.offsets("GoPro MET"));

    Ok(())
}
```