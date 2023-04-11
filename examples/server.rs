#[cfg(target_os = "macos")]
fn main() {
    println!("this is a xdg only feature")
}

#[cfg(target_os = "windows")]
fn main() {
    println!("this is a xdg only feature")
}

#[cfg(all(unix, not(feature = "server"), not(target_os = "macos")))]
fn main() {
    println!("server feature required")
}

#[cfg(all(feature = "server", unix, not(target_os = "macos")))]
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::{select, FutureExt};
    use notify_rust::{
        server::{self, print_notification, ReceivedNotification},
        CloseReason,
    };

    let timeout = std::env::args()
        .nth(1)
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(1);

    std::env::set_var("RUST_LOG", "zbus=trace,server=trace,notify_rust=trace");
    env_logger::init();

    if let Err(error) = server::start_at(
        "example",
        move |received: ReceivedNotification| async move {
            // sleep some time, if the timeout is longer than the timeout of the notification
            // then .channels() will return undefined
            print_notification(&received);
            // async_std::task::sleep(std::time::Duration::from_secs(timeout)).await;
            if let Some((action, closer)) = received.channels() {
                // if received.actions.contains(Action"action") {
                select!(
                    _ = action.send("action".into()).fuse() => (),
                    _ = closer.send(CloseReason::Dismissed).fuse() => {},
                );
                // }
            } else {
                log::warn!("channel upgrade failed, can no longer send action or close")
            }
            //   });

            log::debug!("handler done");
        },
    )
    .await
    {
        log::warn!("failed to start notification server {error}")
    }
    // });

    Ok(())
}
