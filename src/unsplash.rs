use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::env::current_dir;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::copy;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

#[derive(Debug, Deserialize)]
pub struct UnsplashImage {
    pub id: String,
    pub urls: ImageUrls,
    //pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageUrls {
    //pub raw: String,
    pub full: String,
    //pub regular: String,
    //pub small: String,
    //pub thumb: String,
}

#[derive(Debug)]
pub enum Orientation {
    Landscape,
    //Portrait,
    //Squarish,
}

pub struct UnsplashClient {
    client: reqwest::Client,
    access_key: String,
}

impl UnsplashClient {
    pub fn new(access_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, access_key }
    }

    pub async fn get_random_image(
        &self,
        orientation: Option<Orientation>,
    ) -> Result<UnsplashImage, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Client-ID {}", self.access_key))?,
        );

        let mut url = reqwest::Url::parse("https://api.unsplash.com/photos/random")?;

        // Add query parameters
        let mut query_pairs = url.query_pairs_mut();

        // Set minimum dimensions for 4K (3840x2160)
        query_pairs.append_pair("w", "3840");
        query_pairs.append_pair("h", "2160");

        if let Some(orientation) = orientation {
            let orientation_str = match orientation {
                Orientation::Landscape => "landscape",
                //Orientation::Portrait => "portrait",
                //Orientation::Squarish => "squarish",
            };
            query_pairs.append_pair("orientation", orientation_str);
        }

        drop(query_pairs); // Release mutable borrow

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .json::<UnsplashImage>()
            .await?;

        Ok(response)
    }

    pub async fn set_as_wallpaper(&self, image: &UnsplashImage) -> Result<(), Box<dyn Error>> {
        // Get the current working directory
        let current_dir = current_dir()?;

        // Create downloads directory if it doesn't exist
        let download_dir = current_dir.join("downloads");
        std::fs::create_dir_all(&download_dir)?;

        // Generate file path for the image
        let file_name = format!("{}.jpg", image.id);
        let file_path = download_dir.join(&file_name);

        // Download the image
        let response = self
            .client
            .get(&image.urls.full)
            .send()
            .await?
            .bytes()
            .await?;

        // Save the image to file
        let mut file = File::create(&file_path)?;
        copy(&mut response.as_ref(), &mut file)?;

        // Convert absolute path to wide string for Windows API
        let absolute_path = file_path.canonicalize()?;
        let wide_path: Vec<u16> = OsStr::new(absolute_path.to_str().unwrap())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Set wallpaper using Windows API
        unsafe {
            let result = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                wide_path.as_ptr() as *mut _,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );

            if result == 0 {
                return Err("Failed to set wallpaper".into());
            }
        }

        // Stop and restart explorer.exe
        Command::new("powershell")
            .args([
                "-Command",
                "Stop-Process -ProcessName 'explorer'; Start-Process 'explorer.exe'",
            ])
            .output()?;

        Ok(())
    }
}
