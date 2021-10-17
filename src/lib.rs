mod tag;
mod error;

use std::io;
use std::io::{Read,Seek,Write,SeekFrom,Cursor};
use std::fs::File;

use byteorder::{LittleEndian,ReadBytesExt};
use aes::Aes128;
use block_modes::{BlockMode, Ecb, block_padding::Pkcs7};

use base64;
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct NcmMeta {
	format: String,
	#[serde(rename = "musicId")]
	music_id: u32,
	#[serde(rename = "musicName")]
	music_name: String,
	artist: Vec<(String, u32)>,
	album: String,
	#[serde(rename = "albumId")]
	album_id: u32,
	#[serde(rename = "albumPic")]
	album_pic: String,
	// bitrate,duration,alias,mvId
    #[serde(skip)]
	album_pic_type: String,
    #[serde(skip)]
	album_pic_data: Vec<u8>
}

impl From<NcmMeta> for tag::Tag {
    fn from(item: NcmMeta) -> Self {
		let mut t = tag::Tag::new();
		t.name = item.music_name;
		t.album = item.album;
		t.artist = item.artist.into_iter().map(|(n,_)| n).collect();
		t.album_pic_type = item.album_pic_type;
		t.album_pic_data = item.album_pic_data;
		t
    }
}


#[derive(Debug, Default, Clone)]
struct Ncm {
	key_stream: Vec<u8>,
	meta: Vec<u8>,
	image: Vec<u8>,
	data: Vec<u8>,
}

impl Ncm {
	fn new() -> Ncm {
		Ncm { ..Default::default() }
	}

	fn ncm_meta(&self) -> NcmMeta {
		let meta = String::from_utf8_lossy(&self.meta["music:".len()..]).to_string();
		let mut ncm_meta: NcmMeta = serde_json::from_str(&meta).unwrap();
		ncm_meta.album_pic_type = image_type(&self.image).unwrap();
		ncm_meta.album_pic_data = self.image.clone();
		ncm_meta
	}

	fn parse(input: &mut (impl Read + Seek)) -> io::Result<Ncm> {
		fn read(input: &mut impl Read, len: usize) -> io::Result<Vec<u8>> {
			let mut data = vec![0u8;len];
			input.read(&mut data)?;
			Ok(data)
		}
		fn read_i32(input: &mut impl Read) -> io::Result<i32> {
			Ok(input.read_i32::<LittleEndian>()?)
		}
		fn parse_key_stream(mut data: Vec<u8>) -> Vec<u8> {
			// neteasecloudmusic....
			data = xor(data, 0x64).collect();
			let key = decrypt_aes(&CORE_KEY, &mut data).unwrap()[17..].to_vec();
			rc4_keystream(&key)
		}
		fn parse_meta(mut data: Vec<u8>) -> Vec<u8> {
			data = xor(data, 0x63).collect();
			data = base64::decode(&data[22..]).unwrap();
			decrypt_aes(&META_KEY, &mut data).unwrap().to_vec()
		}
		
		let mut ncm = Ncm::new();
		read(input, 8)?;
		input.seek(SeekFrom::Current(2))?;

		ncm.key_stream = {
			let len = read_i32(input)? as usize;
			parse_key_stream(read(input, len)?)
		};
		
		ncm.meta = {
			let len = read_i32(input)? as usize;
			parse_meta(read(input, len)?)
		};
		input.seek(SeekFrom::Current(5))?;

		ncm.image = {
			let image_space = read_i32(input)? as usize;
			let image_size = read_i32(input)? as usize;
			let data = read(input, image_size)?;
			input.seek(SeekFrom::Current((image_space - image_size) as i64))?;
			data
		};
		ncm.data = {
			let mut data = Vec::new();
			input.read_to_end(&mut data)?;
			let key_stream = (0..).map(|i| ncm.key_stream[i % ncm.key_stream.len()]);
			xor_iter(data,  key_stream).collect::<Vec<u8>>()
		};
		Ok(ncm)
	}
}

fn image_type(data: &[u8]) -> Option<String> {
	let r: Option<&str> = match data[..4] {
		[137, 80, 78, 71] => Some("image/png"),
		[0xFF, 0xD8, 0xFF, 0xE0] => Some("image/jpeg"),
		[71, 73, 70, _] => Some("image/gif"),
		_ => None,
	};
	match r {
		Some(s) => Some(s.to_string()),
		None => None
	}
}

const CORE_KEY: [u8;16] = *b"hzHRAmso5kInbaxW";
const META_KEY: [u8;16] = *br"#14ljk_!\]&0U<'(";
const MAGIC_HEADER: [u8;8] = *b"CTENFDAM";

fn xor<I>(iter: I, oper: u8) -> impl Iterator<Item = u8>
where
    I: IntoIterator<Item = u8>,
{
	xor_iter(iter, (0..).map(move |_| oper))
}
fn xor_iter<I1,I2>(iter1: I1, iter2: I2) -> impl Iterator<Item = u8>
where
    I1: IntoIterator<Item = u8>,
    I2: IntoIterator<Item = u8> 
{
	iter1.into_iter().zip(iter2).map(|(x1, x2)| x1^x2)
}

fn decrypt_aes<'a>(key: &[u8], data: &'a mut [u8]) -> Result<&'a [u8], error::AESDecodeError> {
	type Aes128Ecb = Ecb<Aes128, Pkcs7>;
	let cipher = Aes128Ecb::new_from_slices(key, &[])?;
	Ok(cipher.decrypt(data)?)
}

fn rc4_keystream(key: &[u8]) -> Vec<u8> {
	let key_len = key.len();
	let mut s: [usize;256] = [0;256];
	s.iter_mut().enumerate().for_each(|(i, v)| *v = i);

	let mut j:usize = 0;
	for i in 0..256 {
		j = (j + s[i] + key[i % key_len] as usize) & 0xff;
		s.swap(i, j);
	}

	(0..256).map(|i| {
			let i = (i + 1) % 256;
			let sj = s[(i + s[i]) % 256];
			//s.swap(i, j);
			s[(s[i] + sj) % 256] as u8
		})
		.collect()
}

pub fn parse_file(path: &str) -> Vec<u8> {
	let mut fin = File::open(path).unwrap();
	let mut output = Cursor::new(Vec::new());
	parse(&mut fin, &mut output).unwrap();
	output.into_inner()
}

pub fn parse<R,O>(input: &mut R, output: &mut O) -> Result<String, Box<dyn std::error::Error>> 
	where R: Read + Seek,
		  O: Write,
{
	// read ncm
	{
		let data = {
			let ncm = Ncm::parse(input).unwrap();	
			let mut output = Cursor::new(Vec::new());
			let ncm_meta = ncm.ncm_meta();
			tag::rewrite_tag(ncm.data, &mut output, ncm_meta.into());
			output.into_inner()
		};

		output.write(&data)?;
	}
	Ok("".to_string())
}
