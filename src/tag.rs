use metaflac;

use std::io::{Read,Seek,Write,SeekFrom,Cursor};

#[derive(Default,Clone)]
pub struct Tag {
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

pub fn rewrite_tag<O>(input: Vec<u8>, output: &mut O, tag: Tag) 
	where O: Write,
{
	let mut input = Cursor::new(input);
	let mut flac_tag = metaflac::Tag::read_from(&mut input).unwrap();

	flac_tag.add_picture(
		tag.album_pic_type, 
		metaflac::block::PictureType::CoverFront, 
		tag.album_pic_data
	);
	let vbs = flac_tag.vorbis_comments_mut();
	vbs.set_title(vec![tag.name]);
	vbs.set_album(vec![tag.album]);
	vbs.set_artist(tag.artist);

	flac_tag.write_to(output).unwrap();
	let frame = &input.get_ref()[input.position() as usize..];
	output.write_all(frame).unwrap();
}
