use std::fs;

use std::path::PathBuf;
use structopt::StructOpt;


#[derive(StructOpt, Debug)]
#[structopt(name = "rncm")]
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
	let ncm_output = rncm::parse_file(options.input.to_str().unwrap()).unwrap();
	output.set_extension(ncm_output.meta.format);	
	fs::write(output.to_str().unwrap(), ncm_output.data).unwrap();
}
