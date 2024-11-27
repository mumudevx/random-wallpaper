#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/assets/daily_wallpaper_app.ico"); // Optional: add an icon
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
