Experimental Rust crate for iterating MP4 containers. Does not and will not support any kind of media de/encoding.

```rs
use mp4iter::Mp4;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;
    
    // Iterate over atoms. Currently returns `None` on error.
    for atom in mp4.into_iter() {
        println!("{atom:?}")
    }
    // Print duration of MP4 (i.e. longest track)
    println!("{:?}", mp4.duration());

    Ok(())
}
```