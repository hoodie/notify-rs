#![allow(unused_imports, dead_code)]
#[cfg(all(feature = "images", unix, not(target_os = "macos")))]
use notify_rust::Image;
use notify_rust::{Hint, Notification, Urgency::*};

fn freeze(message: &str) {
    println!("{}\n", message);
    let mut _devnull = String::new();
    let _ = std::io::stdin().read_line(&mut _devnull);
}

#[cfg(target_os = "macos")]
fn main() {
    println!("this is an xdg only feature")
}

#[cfg(target_os = "windows")]
fn main() {
    println!("this is an xdg only feature")
}

#[cfg(linux)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Pay close attention to the output of:");
    println!("$ dbus-monitor \"interface=org.freedesktop.Notifications\"");

    freeze("actionicons");
    Notification::new().hint(Hint::ActionIcons(true)).show()?;

    freeze("urgency: low, medium, high");
    Notification::new().hint(Hint::Urgency(Low)).show()?;
    Notification::new().hint(Hint::Urgency(Normal)).show()?;
    Notification::new().hint(Hint::Urgency(Critical)).show()?;

    freeze("category");
    Notification::new()
        .hint(Hint::Category("device.removed".into()))
        .show()?;

    freeze("DesktopEntry");
    Notification::new().hint(Hint::DesktopEntry("firefox".into())).show()?;

    freeze("ImagePath");
    Notification::new()
        .hint(Hint::ImagePath(
            "/usr/share/icons/hicolor/128x128/apps/firefox.png".into(),
        ))
        .show()?;

    freeze("Resident");
    Notification::new().hint(Hint::Resident(true)).show()?;

    freeze("SoundFile");
    Notification::new()
        .hint(Hint::SoundFile("/usr/share/sounds/alsa/Front_Left.wav".to_owned()))
        .hint(Hint::SoundName("system sound".to_owned()))
        .hint(Hint::SuppressSound(false))
        .show()?;

    freeze("Transient");
    Notification::new().hint(Hint::Transient(false)).show()?;

    freeze("X and Y");
    Notification::new().hint(Hint::X(200)).hint(Hint::Y(200)).show()?;

    #[cfg(all(feature = "images", unix, not(target_os = "macos")))]
    {
        freeze("ImageData");
        let mut image_data = vec![0; 128 * 128 * 3];
        for i in 0..128 * 128 * 3 {
            image_data[i] = (i % 256) as u8;
        }
        Notification::new()
            .hint(Hint::ImageData(Image::from_rgb(128, 128, image_data).unwrap()))
            .summary("You should see stripes in this notification");
    }

    Ok(())

    // freeze("Custom");
    // Notification::new().hint(Hint::Custom("foo","bar")).show();
}
