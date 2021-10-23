use metaflac;
use id3;

use std::io;
use std::io::{Read,Seek,Write,SeekFrom,Cursor};


#[derive(Default,Clone)]
pub struct Tag {
    pub format: String,
	pub name: String,
	pub artist: Vec<String>,
	pub album: String,
	pub album_pic_type: String,
	pub album_pic_data: Vec<u8>
}

impl Tag {
	pub fn new() -> Tag {
		Tag { ..Default::default() }
	}
}

pub fn rewrite_tag<O>(input: Vec<u8>, output: &mut O, tag: Tag) -> io::Result<()>
	where O: Write,
{
	let mut input = Cursor::new(input);
    match tag.format.as_str() {
        "flac" => {
            match metaflac::Tag::read_from(&mut input) {
                Ok(mut flac_tag) => {
                    flac_tag.add_picture(
                        tag.album_pic_type, 
                        metaflac::block::PictureType::CoverFront, 
                        tag.album_pic_data
                    );
                    let vbs = flac_tag.vorbis_comments_mut();
                    vbs.set_title(vec![tag.name]);
                    vbs.set_album(vec![tag.album]);
                    vbs.set_artist(tag.artist);

                    match flac_tag.write_to(output) {
                        Err(e) => eprintln!("{:?}", e),
                        Ok(()) => ()
                    };
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            };
        },
        "mp3" => {
            match id3::Tag::read_from(&mut input) {
                Ok(mut mp3_tag) => {
                    mp3_tag.set_album(tag.album);
                    mp3_tag.set_title(tag.name);
                    tag.artist.iter().for_each(|a| mp3_tag.set_artist(a));
                    mp3_tag.add_picture(id3::frame::Picture {
                        mime_type: tag.album_pic_type,
                        picture_type: id3::frame::PictureType::CoverFront,
                        description: "".to_string(),
                        data: tag.album_pic_data
                    });
                    match mp3_tag.write_to(&mut *output, id3::Version::Id3v24) {
                        Err(e) => eprintln!("{:?}", e),
                        Ok(()) => ()
                    };
                },
                Err(e) => eprintln!("{:?}", e)
            };
        },
        other => eprintln!("unsupport format {}", other)
    };

	let frame = &input.get_ref()[input.position() as usize..];
	output.write_all(frame)?;
    Ok(())
}
