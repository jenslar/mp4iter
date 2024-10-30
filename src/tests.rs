#[cfg(test)]
mod tests {
    use crate::{reader::TargetReader, Mp4};
    use std::{
        fs::{create_dir_all, File},
        io::{Read, Seek, SeekFrom, Write},
        path::{Path, PathBuf},
    };

    fn get_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn get_mp4() -> Mp4 {
        // let mut path = get_data_dir();
        // path.push("../mp4iter_dji/data/hero8.mp4"); // gopro hero 8 black
        // path.push("../mp4iter_dji/data/hero5.mp4"); // gopro hero 5 black
        // path.push("../mp4iter_dji/data/DJI_20231202082048_0345_D.MP4"); // dji action osmo 4
        // path.push("../mp4iter_dji/data/VIRB0044.MP4"); // virb ultra 30
        // path.push("../mp4iter_dji/data/DJI_0306.MOV"); // older dji action osmo?

        // work mbp
        // let path = PathBuf::from("/Users/jens/Desktop/PROJECTS/LANGKEY/niclas_burenhult/NB230322/GX010042_RAW/GX010042.MP4");
        // let path = PathBuf::from("/Volumes/jenslar01/mbpbackup231220/Desktop/PROJECTS/DIAD/data/230424_Vång_2305_meeting/jvc01/KMNG0240.MOV"); // failed to fill whole buffer for udta which is 24 bytes, and ends with 4 0-bytes. but mov so perhaps don't support.
        // let path = PathBuf::from("/Users/jens/Desktop/PROJECTS/LANGKEY/niclas_burenhult/gopro11_test/DCIM/100GOPRO/GX010006.MP4");

        // let path = PathBuf::from("/Volumes/jenslar01/data/diad/VV2305/230505/raw/virb_mimma/DCIM/101_VIRB/VIRB0039-2.MP4");
        // let path = PathBuf::from("/Volumes/jenslar01/data/diad/VV2305/230505/raw/insta360/DCIM/Camera01/LRV_20230505_084722_01_001.insv"); // works ok
        // let path = PathBuf::from("/Volumes/jenslar01/data/diad/VV2305/230505/insta360/DCIM/Camera01/VID_20230505_091638_00_002.insv"); // works ok
        // let path = PathBuf::from("/Users/jens/dev/TESTDATA/Video/insta360/VID_20230505_143725_00_014.insv"); // works ok after fix, consistently uses 64-bit atom sizes
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/hero11_heat_battery/230221/DCIM/100GOPRO/GL010019.LRV");
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/dji_osmo_action_4/Set 1 - Mountain Bike/Set1-DJI_20230731113357_0056_D.MP4"); // dji osmo action 4
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/dji_osmo_action_4/Set 2 - Road Bike Descent/Set2-DJI_20230729131927_0040_D.MP4"); // dji osmo action 4
        // let path = PathBuf::from("/Users/jens/Downloads/NightSet1-Action4-DJI_20230905195641_0033_D.MP4"); // dji osmo action 4 home mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/gpmf-parser-master/samples/hero7.mp4"); // GPMF repo sample
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/gpmf-parser-master/samples/max-heromode.mp4"); // home mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/peter_test_videos_hero7_black/GH030026.MP4"); // home mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/hero11_heat_battery/230221/DCIM/100GOPRO/GX010019.MP4"); // work mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/gopro/hero11_heat_battery/230221/DCIM/100GOPRO/GX020019.MP4"); // work mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/VIRB_GPS_TEST/cam04/org/DCIM/102_VIRB/VIRB0592-2.MP4"); // work mbp
        // let path = PathBuf::from("/Users/jens/Desktop/PROJECTS/LANGKEY/niclas_burenhult/gopro11_test/DCIM/100GOPRO/GX010006.MP4"); // work mbp
        // let path = PathBuf::from("/Users/jens/Desktop/DEV/humlab/elan/henrik_jens/ELAN/compose0001_h_3D_720.mp4"); // home mbp
        // let path = PathBuf::from("/Users/jens/Desktop/WORK/LANGKEY/niclas_burenhult/nb230322/GX020006.MP4"); // home mbp
        // let path = PathBuf::from("/Users/jens/Downloads/Set 1 - Mountain Bike/Set1-DJI_20230731113357_0056_D.MP4"); // home mbp
        // let path = PathBuf::from("/Users/jens/Downloads/Set2-DJI_20230729131927_0040_D.MP4"); // home mbp
        let path = PathBuf::from("/Users/jens/dev/TESTDATA/gpmf/GX011369_1727627152283_hero13.mp4"); // hero13

        println!("{}", path.display());
        let mp4_result = Mp4::new(&path);
        if let Err(err) = &mp4_result {
            println!("{err}");
        }
        // println!("{mp4_result:?}");
        assert!(mp4_result.is_ok());
        let mp4 = mp4_result.unwrap();
        println!("read {}", path.display());

        mp4
    }

    #[test]
    fn closest() {
        let ignore_container = false;
        let mut mp4 = get_mp4();
        // let res1 = mp4.seek(SeekFrom::Start(59447942)); // larger than testfile 60_265_380 bytes doesn't raise error on seek, only at read attempt
        let res1 = mp4.seek(SeekFrom::Start(5645073269));
        dbg!(&res1);
        assert!(res1.is_ok());
        if let Ok(pos) = res1 {
            let res2 = mp4
                .reader
                // .header_from_pos(&TargetReader::File, pos, ignore_container);
                .header_closest(&TargetReader::File, Some(pos), ignore_container);
            dbg!(&res2);
            assert!(res2.is_ok());
            if let Ok(hdr) = res2 {
                println!("{hdr:?}");
            }
        }
    }

    #[test]
    fn atoms() {
        let mp4 = get_mp4();
        // let mp4 = get_mp42();

        let mut sizes: Vec<u64> = Vec::new();
        // for header in mp4.headers() {
        for header in mp4.into_iter() {
            let mut pop = false;
            let indent = sizes.len();
            let is_container = header.is_container();
            for size in sizes.iter_mut() {
                if is_container {
                    *size -= 8;
                } else {
                    *size -= header.atom_size;
                }
                if size == &mut 0 {
                    pop = true;
                }
            }

            print!(
                "{}{} @{} size: {} [container = {is_container}]",
                "    ".repeat(indent as usize),
                header.name.to_str(),
                header.offset,
                header.atom_size,
            );
            if is_container {
                sizes.push(header.atom_size - 8);
                print!(" sizes: {:?}", sizes);
            }
            if pop {
                println!(" popping {}", header.name);
                loop {
                    match sizes.last() {
                        Some(&0) => {sizes.pop();}, // '_ = sizes.pop(),' error in zed not in vs code?
                        _ => break,
                    }
                }
            } else {
                println!("");
            }
        }
    }

    #[test]
    fn hdlr() {
        let mut mp4 = get_mp4();
        // let res = mp4.hdlr(false);
        // println!("{res:?}");
        // assert!(res.is_ok());
        while let Ok(hdlr) = mp4.hdlr(false) {
            println!("{hdlr:#?}");
        }
    }

    #[test]
    fn ftyp() {
        let mut mp4 = get_mp4();
        let res = mp4.ftyp(false);
        println!("{res:?}");
        assert!(res.is_ok());
        // while let Ok(ftyp) = mp4.ftyp(false) {
        //     println!("{ftyp:#?}");
        // };
    }

    #[test]
    fn dref() {
        let mut mp4 = get_mp4();
        let res = mp4.dref(false);
        println!("{res:?}");
        assert!(res.is_ok());
        // while let Ok(dref) = mp4.dref(false) {
        //     println!("{dref:#?}");
        // };
    }

    #[test]
    fn stss() {
        let mut mp4 = get_mp4();
        let res = mp4.stss(false);
        println!("{res:?}");
        assert!(res.is_ok());
        // while let Ok(stss) = mp4.dref(false) {
        //     println!("{dref:#?}");
        // };
    }

    #[test]
    fn sdtp() {
        let mut mp4 = get_mp4();
        let res = mp4.sdtp(false);
        println!("{res:?}");
        assert!(res.is_ok());
        // while let Ok(stss) = mp4.dref(false) {
        //     println!("{dref:#?}");
        // };
    }

    #[test]
    fn stts() {
        let mut mp4 = get_mp4();
        // let res = mp4.stts(false);
        // println!("{res:?}");
        // assert!(res.is_ok());
        while let Ok(stts) = mp4.stts(false) {
            println!("{stts:#?}");
            println!("DURATION SUM {}", stts.duration_sum());
            println!("SAMPLE SUM   {}", stts.sample_sum());
        }
    }

    #[test]
    fn stsc() {
        let mut mp4 = get_mp4();
        // let res = mp4.stsc(false);
        // // println!("{res:?}");
        // println!("{res:#?}");
        // assert!(res.is_ok());
        // stsc -> stco, need number of offsets = number of chunks from stco
        while let Ok(stsc) = mp4.stsc(false) {
            println!("track");
            // NOTE: stco does not work, may not even exist on > 4GB MP4...
            //       Perhaps create "virtual" atom combining stco + co64
            //       Also need to find way to test if stco/co64 exists
            //       without accidentally seeking to the next track and
            //       grabbing stco/co64 from there instead...
            let stco = mp4.stco(false).expect("Failed to extract stco");
            // println!("{stsc:#?}");
            // for (i, sample) in stsc.sample_to_chunk_table.iter().enumerate() {
            //     println!("[{}] {} {} {}", i+1, sample.first_chunk, sample.sample_description_id, sample.samples_per_chunk)
            // }
            // println!("{:#?}", stsc.expand());
            println!("  entries {} -> summed via stco {}",
                stsc.no_of_entries,
                // stsc.len(stco.len())
                stsc.len()
            );
            let sum = stsc.sample_to_chunk_table.iter()
                .map(|t| t.samples_per_chunk as usize)
                .sum::<usize>();
            println!("  entries {} -> summed old {sum}",
                stsc.no_of_entries,
            );
        }
    }

    #[test]
    fn tkhd() {
        let mut mp4 = get_mp4();
        // let res = mp4.time_first_frame("GoPro TCD");
        // assert!(res.is_ok());
        // println!("{:?}", res.unwrap());
        let res = mp4.tkhd(true);
        println!("{res:?}");
        assert!(res.is_ok());
        while let Ok(tkhd) = mp4.tkhd(false) {
            println!("{tkhd:#?}");
            println!("width:             {}", tkhd.width());
            println!("height:            {}", tkhd.height());
            println!("duration:          {}", tkhd.duration());
            println!("volume:            {}", tkhd.volume());
            println!("creation_time:     {}", tkhd.creation_time());
            println!("modification_time: {}", tkhd.modification_time());
        }
    }


    #[test]
    fn tkhd_handler() {
        let mut mp4 = get_mp4();

        let mvhd = mp4.mvhd(false);

        println!("{mvhd:?}");

        // gopro
        // let vid = "GoPro AVC";
        let vid = "GoPro H.265";
        let aud = "GoPro AAC";
        let met = "GoPro MET";

        // dji osmo action 4
        // let vid = "VideoHandler";
        // let aud = "SoundHandler";
        // let met = "DJI meta";

        let tkhd_vid = mp4.tkhd_handler(vid, true);
        let tkhd_aud = mp4.tkhd_handler(aud, true);
        let tkhd_met = mp4.tkhd_handler(met, true);

        println!("{vid}: {tkhd_vid:?}\n");
        println!("{aud}: {tkhd_aud:?}\n");
        println!("{met}: {tkhd_met:?}");

    }

    #[test]
    fn mdhd_handler() {
        let mut mp4 = get_mp4();

        let mvhd = mp4.mvhd(false);

        println!("{mvhd:?}");

        // gopro
        // let vid = "GoPro AVC";
        // let aud = "GoPro AAC";
        // let met = "GoPro MET";

        // dji osmo action 4
        let vid = "VideoHandler";
        let aud = "SoundHandler";
        let met = "DJI meta";

        let mdhd_vid = mp4.mdhd_track(vid, true);
        let mdhd_aud = mp4.mdhd_track(aud, true);
        let mdhd_met = mp4.mdhd_track(met, true);

        println!("{vid}: {mdhd_vid:?} {:?}\n", mdhd_vid.as_ref().map(|m| m.language()));
        println!("{aud}: {mdhd_aud:?}\n");
        println!("{met}: {mdhd_met:?}");
    }

    #[test]
    fn first_frame() {
        let mut mp4 = get_mp4();
        // let res1 = mp4.time_first_frame("GoPro H.265", false); // no tmcd in gogpro video track
        // println!("{:?}", res1);
        // assert!(res1.is_ok());
        let res2 = mp4.time_first_frame("GoPro TCD", false);
        println!("{:?}", res2);
        assert!(res2.is_ok());
    }

    // #[test]
    // fn tkhd_handler() {
    //     let mut mp4 = get_mp4();
    //     // let res = mp4.tkhd(false);
    //     // println!("{res:?}");
    //     // assert!(res.is_ok());
    //     let name = "VideoHandler"; // DJI_20231202082048_0345_D.MP4
    //     let result = mp4.tkhd_handler(name, false);
    //     assert!(result.is_ok());
    //     if let Ok(tkhd) = result {
    //         println!("{tkhd:#?}");
    //         println!("width:             {}", tkhd.width());
    //         println!("height:            {}", tkhd.height());
    //         println!("duration:          {}", tkhd.duration());
    //         println!("volume:            {}", tkhd.volume());
    //         println!("creation_time:     {}", tkhd.creation_time());
    //         println!("modification_time: {}", tkhd.modification_time());
    //     };
    // }

    #[test]
    fn udta() {
        let mut mp4 = get_mp4();
        let result = mp4.user_data_headers();
        assert!(result.is_ok());
        if let Ok(headers) = result {
            for header in headers.iter() {
                println!("{header:?}");
                let (pos, len) = (header.offset, header.data_size());
                if let Ok(crs) =
                    mp4.reader
                        .cursor(&TargetReader::Moov, len as usize, Some(SeekFrom::Start(pos)), None)
                        // .cursor_at(&TargetReader::Moov, len as usize, SeekFrom::Start(pos), None)
                {
                    println!("LEN: {}", crs.get_ref().len());
                    // println!("{crs:?}");
                }
            }
        }
    }

    #[test]
    fn user_data_find() {
        let mut mp4 = get_mp4();
        // let result = mp4.find_user_data("Xtra");
        // let result = mp4.find_user_data("AMBA"); // insta360
        let result = mp4.find_user_data("CAME"); // gopro
        assert!(result.is_ok());
        if let Ok(mut atom) = result {
            let bin = atom.read_data().unwrap();
            println!("{bin:?}");
        }
    }

    #[test]
    fn minf_hdlr() {
        let mut mp4 = get_mp4();
        if mp4
            .reader
            .find_header(&TargetReader::Moov, "minf", true)
            .unwrap()
            .is_some()
        {
            let mut atom = mp4.reader.find_atom(&TargetReader::Moov, "hdlr", false).unwrap();
            let hdlr = atom.hdlr().unwrap();
            println!("{hdlr:#?}");
        }
    }

    #[test]
    fn stsd() {
        let mut mp4 = get_mp4();
        loop {
            if let Ok(stsd) = mp4.stsd(false) {
                println!("{stsd:#?}");
                println!("OK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
            } else {
                println!("BREAK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
                break;
            }
        }
    }

    #[test]
    fn video_stsd() {
        let mut mp4 = get_mp4();
        let result = mp4.stsd_video(false);
        assert!(result.is_ok());
        println!("{result:#?}");
    }

    #[test]
    fn tmcd() {
        let mut mp4 = get_mp4();
        // let name = "Garmin AVC"; // Garmin VIRB, but does not have tmcd
        // let name = "Mainconcept Video Media Handler"; // QTM mp4
        // let name = "VideoHandler"; // DJI_20231202082048_0345_D.MP4
        // let name = "TimeCodeHandler"; // DJI_20231202082048_0345_D.MP4
        let name = "GoPro TCD"; // GoPro
                                // let result = mp4.tmcd2(name);
        let result = mp4.tmcd(name, false);
        assert!(result.is_ok());
        println!("{:#?}", result.unwrap());
    }

    #[test]
    fn frame_rate() {
        let mut mp4 = get_mp4();
        let result = mp4.frame_rate();
        assert!(result.is_ok());
        println!("{:#?}", result.unwrap());
    }

    #[test]
    fn attributes() {
        let mut mp4 = get_mp4();
        let major_brand = mp4.major_brand(false);
        println!("major brand: {major_brand:?}");
        let compatible_brands = mp4.compatible_brands(true);
        println!("compatible brands: {compatible_brands:?}");
        let time_scale = mp4.time_scale();
        println!("time scale: {time_scale:?}");
        let resolution = mp4.resolution(true);
        println!("resolution: {resolution:?}");
        let frame_rate = mp4.frame_rate();
        println!("frame rate: {frame_rate:?}");
        let sample_rate = mp4.sample_rate(true);
        println!("sample rate: {sample_rate:?}");
        let video_format = mp4.video_format(true);
        println!("video format: {video_format:?}");
        let audio_format = mp4.audio_format(true);
        println!("audio format: {audio_format:?}");
    }

    #[test]
    fn prefix_black() {
        let mut mp4 = get_mp4();
        let time_scale = mp4.time_scale();
        println!("time scale: {time_scale:?}");
        let resolution = mp4.resolution(true);
        println!("resolution: {resolution:?}");
        let frame_rate = mp4.frame_rate();
        println!("frame rate: {frame_rate:?}");
        let sample_rate = mp4.sample_rate(true);
        println!("sample rate: {sample_rate:?}");
        let video_format = mp4.video_format(true);
        println!("sample rate: {video_format:?}");
    }

    #[test]
    fn mvhd() {
        let mut mp4 = get_mp4();
        loop {
            if let Ok(mvhd) = mp4.mvhd(false) {
                println!("{mvhd:#?}");
                println!("OK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
            } else {
                println!("BREAK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
                break;
            }
        }
    }

    #[test]
    fn vmhd() {
        let mut mp4 = get_mp4();
        loop {
            if let Ok(vmhd) = mp4.vmhd(false) {
                println!("{vmhd:#?}");
                println!("OK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
            } else {
                println!("BREAK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
                break;
            }
        }
    }

    #[test]
    fn stsz() {
        let mut mp4 = get_mp4();
        loop {
            let res = mp4.stsz(false);
            if let Ok(stsz) = res {
                println!("{stsz:#?}");
                println!("OK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
            } else {
                println!("{res:?}");
                println!("BREAK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
                break;
            }
        }
    }

    #[test]
    fn smhd() {
        let mut mp4 = get_mp4();
        loop {
            if let Ok(smhd) = mp4.smhd(false) {
                println!("{smhd:#?}");
                println!("OK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
            } else {
                println!("BREAK @ POS: {:?}", mp4.reader.pos(&TargetReader::Moov));
                break;
            }
        }
    }

    #[test]
    fn track_single() {
        let mut mp4 = get_mp4();
        // let name = "DJI meta";
        let name = "GoPro MET";
        // let name = "GoPro AVC";
        // let name = "GoPro H.265";
        let res = mp4.track(name, false);
        if res.is_err() {
            println!("{res:?}");
        }
        assert!(res.is_ok(), "Failed to extract track details");
        let mut track = res.unwrap();
        // println!("{track:#?}");

        // iter read raw data
        for (i, result) in track.data().enumerate() {
            assert!(result.is_ok(), "Failed to read track data");
            let data = result.unwrap();
            println!("{:04} {} bytes", i+1, data.get_ref().len())
        }
    }

    #[test]
    fn track_list() {
        let mut mp4 = get_mp4();
        let res = mp4.track_list(false);
        // if res.is_err() {
        //     println!("{res:?}");
        // }
        assert!(res.is_ok(), "Failed to extract track info");
        for track in res.unwrap().iter() {
            println!("{} {}", track.name(), track.offsets.len());
        }
    }

    #[test]
    fn datetime() {
        let mut mp4 = get_mp4();
        // println!("{:#?}", mp4.tmcd(handler_name))
        // println!("PATH: {}", mp4.path().display());
        // println!("{:?}", mp4.creation_time(false));
        // println!("{:?}", mp4.duration(true));
        // println!("{:?}", mp4.time_first_frame("VideoHandler", true));
    }

    #[test]
    fn headers() {
        let mut mp4 = get_mp4();
        let now1 = std::time::Instant::now();
        let result1 = mp4.headers();
        let split1 = now1.elapsed();
        assert!(result1.is_ok());
        let headers1 = result1.unwrap();
        let len1 = headers1.len();
        let res = mp4.reset();
        // assert!(zero == 0);
        assert!(res.is_ok());
        let now2 = std::time::Instant::now();
        let headers2 = mp4.into_iter().collect::<Vec<_>>();
        let split2 = now2.elapsed();
        let len2 = headers2.len();
        dbg!(&headers2);
        println!("'headers()':             {split1:?} [len: {len1}]");
        println!("'into_iter().collect()': {split2:?} [len: {len2}]");
        assert!(len1 == len2);
    }

    fn write_data_file(
        name: &str,
        rel_dir: Option<PathBuf>,
        content: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let mut path = get_data_dir().join("data");
        if let Some(p) = rel_dir {
            path.push(p);
        }
        if !path.exists() {
            println!("Creating {}", path.display());
            create_dir_all(&path).expect("Failed to created dirs");
        }
        path.push(name);
        println!("Writing {path:?}");
        let mut file = File::create(&path).unwrap();
        file.write_all(&content)?;
        Ok(path)
    }

    // #[test]
    // fn dump_dji_meta() {
    //     let mut mp4 = get_mp4();
    //     let result = mp4.find_user_data("meta");
    //     assert!(result.is_ok());
    //     if let Ok(mut atom) = result {
    //         let path = get_data_dir();
    //         let p = path.join("data/dji/dji_udta_meta.bin");
    //         let mut file = File::create(&p).unwrap();
    //         let bin = atom.read().unwrap();
    //         let r1 = file.write_all(&bin);
    //         assert!(r1.is_ok());
    //         println!("Wrote {}", p.display());
    //     }
    // }

    // #[test]
    // fn dump_dji_timed() {
    //     let mut mp4 = get_mp4();
    //     let r1 = mp4.offsets("DJI meta");
    //     assert!(r1.is_ok());
    //     if let Ok(offsets) = r1 {
    //         println!("{} offsets", offsets.len());
    //         for (i, offset) in offsets.iter().enumerate() {
    //             let buf = mp4.read_len_at(offset.position, offset.size as u64); // offset.size should be u64?
    //             assert!(buf.is_ok());
    //             let p = write_data_file(&format!("dji_raw_{:04}.bin", i+1), Some(PathBuf::from("dji")), &mut buf.unwrap()).unwrap();
    //             println!("Wrote {}", p.display());
    //         }
    //         // let first = offsets.first().unwrap();
    //         // println!("{first:?}");
    //         // let last = offsets.last().unwrap();
    //         // println!("{last:?}");
    //         // // let r2 = mp4.seek(std::io::SeekFrom::Start(first.position));
    //         // // assert!(r2.is_ok());
    //         // // let mut buf = vec![0_u8; first.size as usize];
    //         // // let r3 = mp4.read_exact(&mut buf);
    //         // // running https://github.com/mildsunrise/protobuf-inspector on resulting files works ok
    //         // // protobuf_inspector < dji_first_timed.bin
    //         // let buf1 = mp4.read_len_at(first.position, first.size as u64); // offset.size should be u64?
    //         // assert!(buf1.is_ok());
    //         // let p1 = write_data_file("dji_first_timed.bin", &mut buf1.unwrap()).unwrap();
    //         // println!("Wrote {}", p1.display());
    //         // let buf2 = mp4.read_len_at(last.position, last.size as u64); // offset.size should be u64?
    //         // assert!(buf2.is_ok());
    //         // let p2 = write_data_file("dji_last_timed.bin", &mut buf2.unwrap()).unwrap();
    //         // println!("Wrote {}", p2.display());
    //     }
    // }

    #[test]
    fn offsets() {
        let mut mp4 = get_mp4();
        mp4.reset().unwrap();
        // let hdlr_name = "DJI meta"; // djmd timed metadata?
        // let hdlr_name = "DJI dbgi"; // debug info ???
        let hdlr_name = "GoPro MET"; // debug info ???
        // let hdlr_name = "GoPro AAC"; // debug info ???
        // let hdlr_name = "GoPro AVC"; // debug info ???
        // let hdlr_name = "GoPro MET"; // debug info ???
                                     // let hdlr_name = "TimeCodeHandler"; // only 4 bytes
        let offsets = mp4.offsets(hdlr_name, false).unwrap();

        for (i, o) in offsets.iter().enumerate() {
            println!("{:6} {hdlr_name} {o:?}", i+1)
        }
        println!("{}", mp4.path().display())
    }

    #[test]
    fn offsets2() {
        let mut mp4 = get_mp4();
        mp4.reset().unwrap();
        // let hdlr_name = "DJI meta"; // djmd timed metadata?
        let hdlr_name = "SoundHandler"; // djmd timed metadata?
        // let hdlr_name = "DJI dbgi"; // debug info ???
        // let hdlr_name = "GoPro MET"; // debug info ???
        // let hdlr_name = "GoPro AAC"; // debug info ???
        // let hdlr_name = "GoPro AVC"; // debug info ???
        // let hdlr_name = "GoPro MET"; // debug info ???
                                     // let hdlr_name = "TimeCodeHandler"; // only 4 bytes
        let offsets = mp4.offsets(hdlr_name, false).unwrap();

        for (i, o) in offsets.iter().enumerate() {
            println!("{:6} {hdlr_name} {o:?}", i+1)
        }
        println!("{}", mp4.path().display())
    }

    // #[test]
    // fn chunks() {
    //     let mut mp4 = get_mp4();
    //     mp4.reset().unwrap();
    //     let hdlr_name = "DJI meta"; // djmd timed metadata?

    //     let cursors = mp4.cursors(hdlr_name, false);
    //     assert!(cursors.is_ok(), "Failed to extract data");
    //     let cursors = cursors.unwrap();
    //     for (i, crs) in cursors.iter().enumerate() {
    //         println!("\n{:6} {crs:?}", i+1)
    //     }
    // }

    // #[test]
    // fn headers() {
    //     let mut mp4 = get_mp4();
    //     let headers = mp4.headers().unwrap();
    //     for h in headers.iter() {
    //         println!("{h:?}");
    //     }
    // }
}
