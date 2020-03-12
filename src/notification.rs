#[cfg(linux)] use dbus::{arg::messageitem::{MessageItem, MessageItemArray}, ffidisp::{Connection, BusType} };

#[cfg(linux)] use crate::xdg::{build_message, NotificationHandle};
#[cfg(linux)] use crate::hints::{Hint, message::HintMessage};
#[cfg(linux)] use crate::urgency::Urgency;
#[cfg(all(unix, not(target_os = "macos"), feature="images"))] use crate::image::Image;

#[cfg(target_os = "windows")] use winrt_notification::Toast;
#[cfg(target_os = "windows")] use std::str::FromStr;
#[cfg(target_os = "windows")] use std::path::Path;

#[cfg(all(unix, target_os = "macos"))] use crate::macos::NotificationHandle;
use crate::timeout::Timeout;
use crate::error::*;

#[cfg(linux)]
use std::collections::HashSet;
use std::default::Default;
use std::env;


// Returns the name of the current executable, used as a default for `Notification.appname`.
fn exe_name() -> String {
    env::current_exe().unwrap()
    .file_name().unwrap().to_str().unwrap().to_owned()
}

/// Desktop notification.
///
/// A desktop notification is configured via builder pattern, before it is launched with `show()`.
///
/// # Example
/// ``` no-run
///     Notification::new()
///         .summary("☝️ A notification")
///         .show()?;
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Notification {
    /// Filled by default with executable name.
    pub appname: String,
    /// Single line to summarize the content.
    pub summary: String,
    /// Subtitle for macOS
    pub subtitle: Option<String>,
    /// Multiple lines possible, may support simple markup,
    /// check out `get_capabilities()` -> `body-markup` and `body-hyperlinks`.
    pub body:    String,
    /// Use a file:// URI or a name in an icon theme, must be compliant freedesktop.org.
    pub icon:    String,
    /// Check out `Hint`
    #[cfg(linux)]
    pub hints:   HashSet<Hint>,
    /// See `Notification::actions()` and `Notification::action()`
    pub actions: Vec<String>,
    #[cfg(target_os="macos")] sound_name: Option<String>,
    #[cfg(target_os="windows")] sound_name: Option<String>,
    #[cfg(target_os="windows")] path_to_image: Option<String>,
    /// Lifetime of the Notification in ms. Often not respected by server, sorry.
    pub timeout: Timeout, // both gnome and galago want allow for -1
    /// Only to be used on the receive end. Use Notification hand for updating.
    pub(crate) id: Option<u32>
}

impl Notification {
    /// Constructs a new Notification.
    ///
    /// Most fields are empty by default, only `appname` is initialized with the name of the current
    /// executable.
    /// The appname is used by some desktop environments to group notifications.
    pub fn new() -> Notification {
        Notification::default()
    }

    /// Overwrite the appname field used for Notification.
    ///
    /// # Platform Support
    /// Please note that this method has no effect on macOS. Here you can only set the application via [`set_application()`](fn.set_application.html)
    pub fn appname(&mut self, appname: &str) -> &mut Notification {
        self.appname = appname.to_owned();
        self
    }

    /// Set the `summary`.
    ///
    /// Often acts as title of the notification. For more elaborate content use the `body` field.
    pub fn summary(&mut self, summary: &str) -> &mut Notification {
        self.summary = summary.to_owned();
        self
    }

    /// Set the `subtitle`.
    ///
    /// This is only useful on macOS, it's not part of the XDG specification and will therefore be eaten by gremlins under your CPU 😈🤘.
    pub fn subtitle(&mut self, subtitle: &str) -> &mut Notification {
        self.subtitle = Some(subtitle.to_owned());
        self
    }

    /// Manual wrapper for `Hint::ImageData`
    #[cfg(all(feature = "images", unix, not(target_os = "macos")))]
    pub fn image_data(&mut self, image: Image) -> &mut Notification {
        self.hint(Hint::ImageData(image));
        self
    }

    /// Wrapper for `Hint::ImagePath`
    #[cfg(linux)]
    pub fn image_path(&mut self, path: &str) -> &mut Notification {
        self.hint(Hint::ImagePath(path.to_string()));
        self
    }

     /// Wrapper for `NotificationHint::ImagePath`
    #[cfg(target_os="windows")]
    pub fn image_path(&mut self, path:&str) -> &mut Notification {
        self.path_to_image = Some(path.to_string());
        self
    }

    /// Wrapper for `Hint::ImageData`
    #[cfg(all(feature = "images", unix, not(target_os = "macos")))]
    pub fn image<T: AsRef<std::path::Path> + Sized>(&mut self, path: T) -> Result<&mut Notification> {
        let img = Image::open(&path)?;
        self.hint(Hint::ImageData(img));
        Ok(self)
    }

    /// Wrapper for `Hint::SoundName`
    #[cfg(linux)]
    pub fn sound_name(&mut self, name: &str) -> &mut Notification {
        self.hint(Hint::SoundName(name.to_owned()));
        self
    }

    /// Set the sound_name for the NSUserNotification
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn sound_name(&mut self, name: &str) -> &mut Notification {
        self.sound_name = Some(name.to_owned());
        self
    }

    /// Set the content of the `body` field.
    ///
    /// Multiline textual content of the notification.
    /// Each line should be treated as a paragraph.
    /// Simple html markup should be supported, depending on the server implementation.
    pub fn body(&mut self, body: &str) -> &mut Notification {
        self.body = body.to_owned();
        self
    }

    /// Set the `icon` field.
    ///
    /// You can use common icon names here, usually those in `/usr/share/icons`
    /// can all be used.
    /// You can also use an absolute path to file.
    ///
    /// # Platform support
    /// macOS does not have support manually setting the icon. However you can pretend to be another app using [`set_application()`](fn.set_application.html)
    pub fn icon(&mut self, icon: &str) -> &mut Notification {
        self.icon = icon.to_owned();
        self
    }

    /// Set the `icon` field automatically.
    ///
    /// This looks at your binary's name and uses it to set the icon.
    ///
    /// # Platform support
    /// macOS does not support manually setting the icon. However you can pretend to be another app using [`set_application()`](fn.set_application.html)
    pub fn auto_icon(&mut self) -> &mut Notification {
        self.icon = exe_name();
        self
    }

    /// Adds a hint.
    ///
    /// This method will add a hint to the internal hint HashSet.
    /// Hints must be of type `Hint`.
    ///
    /// Many of these are again wrapped by more convenient functions such as:
    ///
    /// * `sound_name(...)`
    /// * `urgency(...)`
    /// * [`image(...)`](#method.image) or
    ///   * [`image_data(...)`](#method.image_data)
    ///   * [`image_path(...)`](#method.image_path)
    ///
    /// ```no_run
    /// # use notify_rust::Notification;
    /// # use notify_rust::Hint;
    /// Notification::new().summary("Category:email")
    ///                    .body("This should not go away until you acknowledge it.")
    ///                    .icon("thunderbird")
    ///                    .appname("thunderbird")
    ///                    .hint(Hint::Category("email".to_owned()))
    ///                    .hint(Hint::Resident(true))
    ///                    .show();
    /// ```
    ///
    /// # Platform support
    /// Most of these hints don't even have an effect on the big XDG Desktops, they are completely tossed on macOS.
    #[cfg(linux)]
    pub fn hint(&mut self, hint: Hint) -> &mut Notification {
        self.hints.insert(hint);
        self
    }

    /// Set the `timeout`.
    ///
    /// This sets the time (in milliseconds) from the time the notification is displayed until it is
    /// closed again by the Notification Server.
    /// According to [specification](https://developer.gnome.org/notification-spec/)
    /// -1 will leave the timeout to be set by the server and
    /// 0 will cause the notification never to expire.
    ///
    /// # Platform support
    /// This only works on XDG Desktops, macOS does not support manually setting the timeout.
    pub fn timeout<T: Into<Timeout>>(&mut self, timeout: T) -> &mut Notification {
        self.timeout = timeout.into();
        self
    }

    /// Set the `urgency`.
    ///
    /// Pick between Medium, Low and High.
    ///
    /// # Platform support
    /// Most Desktops on linux and bsd are far too relaxed to pay any attention to this.
    /// In macOS this does not exist
    #[cfg(linux)]
    pub fn urgency(&mut self, urgency: Urgency) -> &mut Notification {
        self.hint(Hint::Urgency(urgency)); // TODO impl as T where T: Into<Urgency>
        self
    }

    /// Set `actions`.
    ///
    /// To quote http://www.galago-project.org/specs/notification/0.9/x408.html#command-notify
    ///
    /// >  Actions are sent over as a list of pairs.
    /// >  Each even element in the list (starting at index 0) represents the identifier for the action.
    /// >  Each odd element in the list is the localized string that will be displayed to the user.
    ///
    /// There is nothing fancy going on here yet.
    /// **Careful! This replaces the internal list of actions!**
    ///
    /// (xdg only)
    #[deprecated(note = "please use .action() only")]
    pub fn actions(&mut self, actions: Vec<String>) -> &mut Notification {
        self.actions = actions;
        self
    }

    /// Add an action.
    ///
    /// This adds a single action to the internal list of actions.
    ///
    /// (xdg only)
    pub fn action(&mut self, identifier: &str, label: &str) -> &mut Notification {
        self.actions.push(identifier.to_owned());
        self.actions.push(label.to_owned());
        self
    }

    /// Set an Id ahead of time
    ///
    /// Setting the id ahead of time allows overriding a known other notification.
    /// Though if you want to update a notification, it is easier to use the `update()` method of
    /// the `NotificationHandle` object that `show()` returns.
    ///
    /// (xdg only)
    pub fn id(&mut self, id: u32) -> &mut Notification {
        self.id = Some(id);
        self
    }

    /// Finalizes a Notification.
    ///
    /// Part of the builder pattern, returns a complete copy of the built notification.
    pub fn finalize(&self) -> Notification {
        self.clone()
    }

    #[cfg(linux)]
    fn pack_hints(&self) -> Result<MessageItem> {
        if !self.hints.is_empty() {
            let hints = self.hints
                .iter()
                .cloned()
                .map(HintMessage::wrap_hint)
                .collect::<Vec<(MessageItem, MessageItem)>>();

            if let Ok(array) = MessageItem::new_dict(hints) {
                return Ok(array);
            }
        }

        Ok(MessageItem::Array(MessageItemArray::new(vec![], "a{sv}".into()).unwrap()))
    }

    #[cfg(linux)]
    fn pack_actions(&self) -> MessageItem {
        if !self.actions.is_empty() {
            let mut actions = vec![];
            for action in &self.actions {
                actions.push(action.to_owned().into());
            }
            if let Ok(array) = MessageItem::new_array(actions) {
                return array;
            }
        }

        MessageItem::Array(MessageItemArray::new(vec![], "as".into()).unwrap())
    }

    /// Sends Notification to D-Bus.
    ///
    /// Returns a handle to a notification
    #[cfg(linux)]
    pub fn show(&self) -> Result<NotificationHandle> {
        let connection = Connection::get_private(BusType::Session)?;
        let inner_id = self.id.unwrap_or(0);
        let id = self._show(inner_id, &connection)?;
        Ok(NotificationHandle::new(id, connection, self.clone()))
    }

    /// Sends Notification to NSUserNotificationCenter.
    ///
    /// Returns an `Ok` no matter what, since there is currently no way of telling the success of
    /// the notification.
    #[cfg(target_os = "macos")]
    pub fn show(&self) -> Result<NotificationHandle> {
        mac_notification_sys::send_notification(
            &self.summary, //title
            &self.subtitle.as_ref().map(|s| &**s), // subtitle
            &self.body, //message
            &self.sound_name.as_ref().map(|s| &**s) // sound
        )?;

        Ok(NotificationHandle::new(self.clone()))
    }

     /// Sends Notification to NSUserNotificationCenter.
    ///
    /// Returns an `Ok` no matter what, since there is currently no way of telling the success of
    /// the notification.
    #[cfg(target_os = "windows")]
    pub fn show(&self) -> Result<()> {
        let sound_name = self.sound_name.clone();
        let sound = match sound_name {
            Some(chosen_sound_name) => winrt_notification::Sound::from_str(&chosen_sound_name).ok(),
            None => None
        };

        let duration = match self.timeout {
            Timeout::Default => winrt_notification::Duration::Short,
            Timeout::Never => winrt_notification::Duration::Long,
            Timeout::Milliseconds(t) => if t >= 25000 {
                winrt_notification::Duration::Long
            } else {
                winrt_notification::Duration::Short
            }
        };

        let mut toast = Toast::new(Toast::POWERSHELL_APP_ID) //Not using app name due winrt-notification#1
            .title(&self.summary)
            .text1(&self.subtitle.as_ref().map(|s| &**s).unwrap_or("")) // subtitle
            .text2(&self.body)
            .sound(sound)
            .duration(duration);
        if let Some(image_path) = &self.path_to_image {
            toast = toast.image(&Path::new(&image_path), "");
        }

        toast.show()
            .map_err(|e| {
                Error::from(ErrorKind::Msg(format!("{:?}",e)))
            })
    }

    #[cfg(linux)]
    pub(crate) fn _show(&self, id: u32, connection: &Connection) -> Result<u32> {
        let mut message = build_message("Notify");
        let timeout: i32 = self.timeout.into();
        message.append_items(&[self.appname.to_owned().into(), // appname
                               id.into(),                      // notification to update
                               self.icon.to_owned().into(),    // icon
                               self.summary.to_owned().into(), // summary (title)
                               self.body.to_owned().into(),    // body
                               self.pack_actions(),            // actions
                               self.pack_hints()?,             // hints
                               timeout.into()                  // timeout
        ]);

        let reply = connection.send_with_reply_and_block(message, 2000)?;

        match reply.get_items().get(0) {
            Some(&MessageItem::UInt32(ref id)) => Ok(*id),
            _ => Ok(0)
        }
    }

    /// Wraps show() but prints notification to stdout.
    #[cfg(linux)]
    pub fn show_debug(&mut self) -> Result<NotificationHandle> {
        println!("Notification:\n{appname}: ({icon}) {summary:?} {body:?}\nhints: [{hints:?}]\n",
                 appname = self.appname,
                 summary = self.summary,
                 body = self.body,
                 hints = self.hints,
                 icon = self.icon,);
        self.show()
    }
}

impl Default for Notification {
    #[cfg(linux)]
    fn default() -> Notification {
        Notification {
            appname:  exe_name(),
            summary:  String::new(),
            subtitle: None,
            body:     String::new(),
            icon:     String::new(),
            hints:    HashSet::new(),
            actions:  Vec::new(),
            timeout:  Timeout::Default,
            id:       None
        }
    }

    #[cfg(target_os = "macos")]
    fn default() -> Notification {
        Notification {
            appname:    exe_name(),
            summary:    String::new(),
            subtitle:   None,
            body:       String::new(),
            icon:       String::new(),
            actions:    Vec::new(),
            timeout:    Timeout::Default,
            sound_name: Default::default(),
            id:         None
        }
    }

    #[cfg(target_os="windows")]
    fn default() -> Notification {
        Notification {
            appname:  exe_name(),
            summary:  String::new(),
            subtitle:  None,
            body:     String::new(),
            icon:     String::new(),
            actions:  Vec::new(),
            timeout:  Timeout::Default,
            sound_name: Default::default(),
            id:       None,
            path_to_image: None
        }
    }
}

