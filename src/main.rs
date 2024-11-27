mod unsplash;

use dotenv::dotenv;
use std::env;
use unsplash::Orientation;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let access_key =
        env::var("UNSPLASH_ACCESS_KEY").expect("UNSPLASH_ACCESS_KEY must be set in environment");

    let client = unsplash::UnsplashClient::new(access_key);

    match client.get_random_image(Some(Orientation::Landscape)).await {
        Ok(image) => {
            println!("Downloading image: {}", image.urls.full);
            match client.set_as_wallpaper(&image).await {
                Ok(_) => println!("Wallpaper set successfully!"),
                Err(e) => eprintln!("Error setting wallpaper: {}", e),
            }
        }
        Err(e) => eprintln!("Error fetching image: {}", e),
    }
}
