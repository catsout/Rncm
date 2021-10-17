use std::fs;

use std::path::PathBuf;
use structopt::StructOpt;


#[derive(StructOpt, Debug)]
#[structopt(name = "rncmdump")]
struct Opt {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
	
    #[structopt(required = true, parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let options = Opt::from_args();
	let mut output: PathBuf = match options.output {
		Some(p) => p,
		None => options.input.clone()
	};
	let ncm_output = rncmdump::parse_file(options.input.to_str().unwrap());
	output.set_extension("flac");	
	fs::write(output.to_str().unwrap(), ncm_output).unwrap();
}
