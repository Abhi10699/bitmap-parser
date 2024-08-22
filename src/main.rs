use std::path::Path;

use bmp_parser::bmp::BMP;

fn main() {
    let image_file_path = Path::new("files/image.bmp");
    let bmp = BMP::read(image_file_path);
    bmp.print_metadata();
}
