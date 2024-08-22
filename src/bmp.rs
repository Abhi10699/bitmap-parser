use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
    u32, usize, vec,
};

use serde::{Deserialize, Serialize};

use crate::utils::{combine_8bits, combine_8bits_signed};

#[derive(Debug, Deserialize, Serialize)]
pub struct BMP {
    file_identifier: String,
    file_size_bytes: u32,

    pixel_read_addr: u32,
    dib_header_size: u32,
    image_width: i32,
    image_height: i32,

    num_color_planes: u16,
    bits_per_pixel: u16,

    compression_method: u32,
    compression_method_name: String,
    raw_image_size: u32,
    resolution_horizontal_ppm: i32,
    resolution_vertical_ppm: i32,
    num_colors: u32,
    num_important_colors: u32,

    // pixel array
    pixel_arr_flat: Vec<u8>,
}

impl BMP {
    pub fn read(path: &Path) -> BMP {
        let mut image_file = File::open(path).expect("Error opening file");
        let mut bmp = BMP {
            // File Header
            file_identifier: String::new(),
            file_size_bytes: 0,
            pixel_read_addr: 0,

            // DIB Header
            dib_header_size: 0,
            image_height: 0,
            image_width: 0,
            num_color_planes: 0,
            bits_per_pixel: 0,
            compression_method: 0,
            compression_method_name: String::new(),
            num_colors: 0,
            num_important_colors: 0,
            raw_image_size: 0,
            resolution_horizontal_ppm: 0,
            resolution_vertical_ppm: 0,

            // Pixel Array
            pixel_arr_flat: Vec::new(),
        };

        bmp.read_file_header(&mut image_file);
        bmp.read_dib_header(&mut image_file);
        bmp.read_pixel_array(&mut image_file);
        return bmp;
    }

    pub fn print_metadata(self) {
        println!("File Identifier: {}", self.file_identifier);
        println!("File Size (bytes): {}", self.file_size_bytes);
        println!("Pixel Array Address: {}", self.pixel_read_addr);
        println!("DIB Header Size: {}", self.dib_header_size);
        println!("Image Height: {}", self.image_height);
        println!("Image Width: {}", self.image_width);
        println!("Number of Color Planes: {}", self.num_color_planes);
        println!("Bits Per Pixel: {}", self.bits_per_pixel);
        println!("Compression Method: {}", self.compression_method_name);
        println!("# of Colors: {}", self.num_colors);
        println!("# of important Colors: {}", self.num_important_colors);
        println!("Raw Image Size: {}", self.raw_image_size);
        println!(
            "Horizontal Resolution PPM: {}",
            self.resolution_horizontal_ppm
        );
        println!("Vertical Resolution PPM: {}", self.resolution_vertical_ppm);
        println!("Pixel Array Size: {}", self.pixel_arr_flat.len())
    }

    fn read_file_header(&mut self, image_file: &mut File) {
        const FILE_HEADER_LEN: usize = 14;
        let mut header_buffer = [0; FILE_HEADER_LEN];
        let header_bytes_read = image_file
            .read(&mut header_buffer)
            .expect("Error reading the header");

        if header_bytes_read != 14 {
            panic!("Bytes Count Mismatch, Expected 14 found {header_bytes_read}")
        }

        // Parse first Identifier Byte
        for byte in 0..2 {
            let c = char::from_u32(header_buffer[byte] as u32)
                .expect("Error converting decimal to ascii");
            self.file_identifier.push(c);
        }

        // Parse file size

        self.file_size_bytes = combine_8bits(&header_buffer[2..6]);

        // Parse pixel starting location

        self.pixel_read_addr = combine_8bits(&header_buffer[10..14])
    }

    fn read_dib_header(&mut self, image_file: &mut File) {
        // seek to next n bytes
        const DIB_HEADER_LEN: usize = 40;

        let mut dib_header_buffer = [0; DIB_HEADER_LEN];
        let dib_header_byte_read = image_file.read(&mut dib_header_buffer).unwrap();

        if dib_header_byte_read != DIB_HEADER_LEN {
            panic!("Bytes Count Mismatch, Expected {DIB_HEADER_LEN} found {dib_header_byte_read}")
        }

        self.dib_header_size = combine_8bits(&dib_header_buffer[0..4]);
        self.image_width = combine_8bits_signed(&dib_header_buffer[4..8]);
        self.image_height = combine_8bits_signed(&dib_header_buffer[8..12]);

        self.num_color_planes = combine_8bits(&dib_header_buffer[12..14]) as u16;

        self.bits_per_pixel = combine_8bits(&dib_header_buffer[14..16]) as u16;

        self.compression_method = combine_8bits(&dib_header_buffer[16..20]);

        self.compression_method_name = match self.compression_method {
            0 => "none",
            1 => "RLE 8-bit/pixel",
            2 => "RLE 4-bit/pixel",
            3 => "Huffman 1D",
            4 => "RLE-24",
            5 => "",
            6 => "RGBA bit field masks",
            11 => "none",
            12 => "RLE-8",
            13 => "RLE-4",
            _ => "",
        }
        .to_string();

        self.raw_image_size = combine_8bits(&dib_header_buffer[20..24]);
        self.resolution_horizontal_ppm = combine_8bits_signed(&dib_header_buffer[24..28]);
        self.resolution_vertical_ppm = combine_8bits_signed(&dib_header_buffer[28..32]);
        self.num_colors = combine_8bits(&dib_header_buffer[32..36]);
        self.num_important_colors = combine_8bits(&dib_header_buffer[36..40]);
    }

    fn read_pixel_array(&mut self, image_file: &mut File) {
        let row_size = (((self.bits_per_pixel * self.image_width as u16) + 31) / 32) * 4;
        let bytes_per_pixel = self.bits_per_pixel / 8;
        let padding_size = (self.image_width * bytes_per_pixel as i32) % 4;
        let actual_read_size = row_size - padding_size as u16;

        let mut pixel_stack: Vec<Vec<u8>> = Vec::new();

        for _ in 0..self.image_height {
            let mut pixel_buffer = vec![0; actual_read_size as usize];
            let _ = image_file.read(&mut pixel_buffer).unwrap();

            pixel_stack.push(pixel_buffer);

            // skip padding bytes
            let _ = image_file.seek(std::io::SeekFrom::Current(padding_size as i64));
        }

        for _ in 0..pixel_stack.len() {
            let bgr_vals = pixel_stack.pop().unwrap();

            let pixel_counter = bgr_vals.chunks(3).map(|x| convert_bgr_to_rgba(x, 255));

            for i in pixel_counter {
                self.pixel_arr_flat.extend(i);
            }
        }
    }
}

fn convert_bgr_to_rgba(bgr: &[u8], alpha: u8) -> Vec<u8> {
    vec![bgr[2], bgr[1], bgr[0], alpha]
}
