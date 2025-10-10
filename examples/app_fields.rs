/// Example showing App struct has all fields
use openrtb_rs::openrtb::bid_request::App;

fn main() {
    let app = App {
        id: "app-123".to_string(),
        name: "My Awesome App".to_string(),
        bundle: "com.example.app".to_string(),
        storeurl: "https://play.google.com/store/apps/details?id=com.example.app".to_string(),
        ver: "1.2.3".to_string(),
        ..Default::default()
    };

    println!("App ID: {}", app.id);
    println!("App Name: {}", app.name);
    println!("Bundle: {}", app.bundle);
    println!("Store URL: {}", app.storeurl);
    println!("Version: {}", app.ver);

    println!("\n✅ App struct has all fields!");
}
