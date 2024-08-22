use bmp_parser::bmp::BMP;
use rocket::fs::FileServer;
use std::path::Path;

#[macro_use]
extern crate rocket;

#[get("/bmp")]
fn get_bmp() -> String {
    let image_file_path = Path::new("files/image.bmp");
    let bmp = BMP::read(image_file_path);
    serde_json::to_string(&bmp).expect("Error converting to JSON")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from("res/"))
        .mount("/api", routes![get_bmp])
}
