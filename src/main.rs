mod converter;

use std::{path::Path, fs};
use xfbin::{xfbin::XfbinPage, Xfbin};
use xfbin::write_xfbin;
use converter::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let filename = Path::new(&args[1]).file_stem().unwrap().to_str().unwrap();

    if !args[1].ends_with(".xml") {
        panic!("File is not an xml file");
    }

    let binding = fs::read_to_string(&args[1]).unwrap();

    let xml = binding.as_str();

    let mut xfbin = Xfbin::default();

    let frame_settings = get_frame_settings(xml);

    create_fcv_xfbin(&mut xfbin, &frame_settings, filename);
    
    
    write_xfbin(xfbin, &Path::new(&format!("{}.fcv.xfbin", filename))).unwrap();
}
