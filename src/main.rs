use dirs::home_dir;
use regex::Regex;
use std::process::Command;
use std::{
    fs,
    path::{Path, PathBuf},
};

use gtk::prelude::*;
use relm4::prelude::*;

/// The type of messages the model uses for input communication
#[derive(Debug)]
enum AppMsg {
    SearchChanged(String),
    MimeSelected(u32),
    AppSet(String),
}

/// The mode of interaction
#[derive(Debug)]
enum AppMode {
    Selecting,
    Searching,
    Setting,
}

/// The model and its state variables
struct AppModel {
    mimes_apps: Vec<(String, Vec<String>)>, // the vector of tuple of mimes and their apps
    search_str: String,                     // what the user searches
    selected_mime_index: u32,               // the index of selected mime in the current dropdown
    selected_mime: String,                  // the selected mime
    current_list: gtk::StringList,          // the current items in the current dropdown
    mode: AppMode,                          // is the user searching? or selecting? or setting?
}

/// The widgets
struct AppWidgets {
    label: gtk::Label,       // the label that updates when searching
    dropdown: gtk::DropDown, // the dropdown containing [narrowed-down] list of mimes
    appbox: gtk::Box,        // the Box for listing apps for the selected mime
}

/// Visualize the variables using Component trait
impl Component for AppModel {
    // the type of message this component will receive
    type Input = AppMsg;
    // the type of message this component will send
    type Output = ();
    // the command output for this component
    type CommandOutput = ();
    // the type of data with which this component will be initialized
    // which is a vector of filepaths containing mimes and apps
    type Init = Vec<PathBuf>;
    // the root GTK window this component will create
    type Root = gtk::Window;
    // the widgets this component will need to update
    type Widgets = AppWidgets;

    /// Create the Root widget i.e., the window widget
    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("xdg mimer")
            .default_width(800)
            .default_height(600)
            .build()
    }

    /// Create the UI and model
    fn init(
        mime_paths: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mimes_apps = get_mimes_apps(mime_paths);
        let selected_mime_index = 0;
        let selected_mime = String::from("");
        let mode = AppMode::Searching; // the default mode
        let search_str = String::from("");

        // a trick to make the dropdown look unselected
        let mut mimes_list = vec!["Select a mime ..."];
        let mimes_list_base: Vec<&str> = mimes_apps.iter().map(|st| &st.0[..]).collect();
        mimes_list.extend(mimes_list_base);
        let dropdown = gtk::DropDown::from_strings(&mimes_list);
        let current_list = gtk::StringList::new(&mimes_list);
        // dropdown.set_selected(gtk::ffi::GTK_INVALID_LIST_POSITION); // no effect
        let model = AppModel {
            search_str,
            mimes_apps,
            mode,
            selected_mime_index,
            selected_mime,
            current_list,
        };

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search the mime type..."));

        let label = gtk::Label::new(Some("Search for a mime, or select one."));

        // connect to signal when the user searches
        let sender_clone = sender.clone();
        search_entry.connect_changed(move |entry| {
            sender_clone.input(AppMsg::SearchChanged(entry.text().to_string()));
        });

        // connect to signal when the user selects from the dropdown
        let sender_clone = sender.clone();
        dropdown.connect_selected_notify(move |dd| {
            sender_clone.input(AppMsg::MimeSelected(dd.selected()));
        });

        // the outer box for holding all the UI elements
        let main_vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(20)
            .build();

        window.set_child(Some(&main_vbox));
        main_vbox.set_margin_all(25);

        // first VBox (containing label, search entry, and dropdown)
        let top_vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        top_vbox.append(&label);
        label.set_margin_vertical(25);
        top_vbox.append(&search_entry);
        top_vbox.append(&dropdown);
        top_vbox.set_margin_bottom(40);

        // second VBox (containing appbox: for the selected mime shows rows of apps and a button)
        let bottom_vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        // a Box as part of AppWidgets for easy reference in update_view
        let appbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(20)
            .build();

        appbox.set_homogeneous(true);

        bottom_vbox.append(&appbox);
        bottom_vbox.set_margin_all(100);
        bottom_vbox.set_margin_top(0);

        // append both top and bottom vboxes to the main box
        main_vbox.append(&top_vbox);
        main_vbox.append(&bottom_vbox);

        let widgets = AppWidgets {
            label,
            dropdown,
            appbox,
        };

        ComponentParts { model, widgets }
    }

    /// Process the messages and update the AppModel
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _window: &Self::Root) {
        match msg {
            AppMsg::SearchChanged(text) => {
                self.mode = AppMode::Searching;
                println!("[{:?}] You searched for: {}", self.mode, &text);
                self.search_str = text;

                let regex =
                    Regex::new(&format!(r"(?i){}", regex::escape(&self.search_str))).unwrap();

                // populate the dropdown items with the regex search results
                let new_list: Vec<&str> = self
                    .mimes_apps
                    .iter()
                    .filter(|st| regex.is_match(&st.0))
                    .map(|st| &st.0[..])
                    .collect();

                self.current_list = gtk::StringList::new(&new_list);
            }
            AppMsg::MimeSelected(indx) => {
                self.mode = AppMode::Selecting;
                self.selected_mime_index = indx;
                if indx == u32::MAX {
                    println!("[{:?}] The search didn't find anything.", self.mode);
                } else {
                    if let Some(mime) = self
                        .current_list
                        .string(indx)
                        .map(|gstring| gstring.to_string())
                    {
                        self.selected_mime = mime;
                    }
                    println!(
                        "[{:?}] You selected item: {:?}",
                        self.mode, self.selected_mime
                    );
                }
            }
            AppMsg::AppSet(app) => {
                self.mode = AppMode::Setting;
                set_default_handler(&self.selected_mime, &app);
            }
        }
    }

    /// Update the view to show the updated model
    /// We update the view based on different variants of AppMode.
    /// This avoids an infinite loop as well.
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // in any mode this state is correct and informative.
        if self.selected_mime_index == u32::MAX {
            widgets
                .label
                .set_label("Your search doesn't match any mime.");
        }

        if let AppMode::Searching = self.mode {
            // in Searching mode we intentionally leave appbox intact
            // (which may contain apps from a previous search)
            if self.search_str == "" {
                widgets
                    .label
                    .set_label("Search for a mime, or directly select one from the dropdown menu.");
            } else {
                widgets
                    .label
                    .set_label(&format!("You searched: {}", self.search_str));
            }
            widgets.dropdown.set_model(Some(&self.current_list));
            // select first matching item (just in case),
            // and to trigger Selecting mode.
            widgets.dropdown.set_selected(0);
        } else {
            // both other AppModes trigger this part.
            // first, remove all the widgets from appbox
            let children = widgets.appbox.observe_children();
            let nbox = children.n_items();
            let to_remove: Vec<gtk::Widget> = (0..nbox)
                .filter_map(|i| children.item(i))
                .filter_map(|obj| obj.downcast::<gtk::Widget>().ok())
                .collect();
            to_remove.iter().for_each(|w| widgets.appbox.remove(w));

            // then, populate the appbox with rows of mime labels and set default buttons
            if let Some(indx) = self
                .mimes_apps
                .iter()
                .position(|mapp| *mapp.0 == self.selected_mime)
            {
                // choose the corresponding vector of apps for the selected mime
                let m_apps = &self.mimes_apps[indx].1;
                println!("[{:?}] The list of apps are: {:?}", self.mode, m_apps);
                // query the default app for the selected mime
                let default_app = get_default_handler(&self.selected_mime);
                println!(
                    "[{:?}] The default app for {:?} is {:?}",
                    self.mode, self.selected_mime, default_app
                );

                // a title for the available apps for the selected mime
                let app_title = gtk::Label::new(None);

                // using pango markup with bold font and 110% of normal size
                app_title.set_markup(&format!(
                    "<span font_weight=\"bold\" size=\"110%\">Available applications for \"{}\"</span>",
                    self.selected_mime
                ));
                widgets.appbox.append(&app_title);

                m_apps.iter().for_each(|app| {
                    let set_default;
                    if default_app.trim() == app.trim() {
                        set_default = gtk::Button::builder().label("Default").build();
                        set_default.set_sensitive(false); // it's already default.
                    } else {
                        set_default = gtk::Button::builder().label("Set as Default").build();
                    }

                    // a horizontal box for holding each app and its button
                    let app_vbox = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .spacing(10)
                        .build();

                    let app_label = gtk::Label::new(Some(&app));
                    app_label.set_hexpand(true); // let the label expand
                    app_label.set_halign(gtk::Align::Start); // align label to start (left)
                    set_default.set_halign(gtk::Align::End); // align button to end (right)

                    app_vbox.append(&app_label);
                    app_vbox.append(&set_default);
                    widgets.appbox.append(&app_vbox);

                    // connect to signal when the button is clicked
                    // this sends an AppMsg input and triggers another
                    // update -> update_view cycle which automatically
                    // updates our view of set default buttons as well.
                    let sender_clone = sender.clone();
                    let app_cloned = app.clone();
                    set_default.connect_clicked(move |_| {
                        sender_clone.input(AppMsg::AppSet(app_cloned.clone()));
                    });
                });
            }
        }
    }
}

//************************Other Helper Functions***********************

/// Gets and stores all mimes and their available apps from a vector of filepaths
fn get_mimes_apps<T: AsRef<Path>>(filenames: Vec<T>) -> Vec<(String, Vec<String>)> {
    // we use HashMap to efficiently update vector of apps for a mime
    // from different sources.
    use std::collections::HashMap;
    let reg = Regex::new(r"^(.*?)=(.*);$").unwrap();
    let mut mime_apps: HashMap<String, Vec<String>> = HashMap::new();
    for filename in filenames {
        // we've already checked their existence before passing to this function
        let file = fs::read_to_string(filename).expect("No such file.");

        file.lines().for_each(|line| {
            if let Some(regmatch) = reg.captures(&line) {
                let key: String = regmatch[1].trim().to_string();
                let mut newval: Vec<String> = regmatch[2]
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();

                mime_apps
                    .entry(key)
                    .and_modify(|v| v.append(&mut newval))
                    .or_insert(newval);
            }
        });
    }

    // at this point the hashmap may have duplicates in its values
    mime_apps.values_mut().for_each(|a| {
        a.sort();
        a.dedup();
    });

    let mut mime_apps: Vec<(String, Vec<String>)> = mime_apps.into_iter().collect();
    mime_apps.sort_by(|a, b| a.0.cmp(&b.0)); // sort by mimes
    mime_apps
}

/// Gets the default app for a given mime
fn get_default_handler(mime_type: &str) -> String {
    // we want it to output the default app if there
    // is one set, otherwise give an empty string for
    // all other cases.
    let output = Command::new("xdg-mime")
        .args(["query", "default", mime_type])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    output
}

/// Sets the default app for a given mime
fn set_default_handler(mime_type: &str, handler: &str) {
    let _ = Command::new("xdg-mime")
        .args(["default", handler, mime_type])
        .status();
}

//*********************************************************************

fn main() {
    let mut mime_paths = Vec::new();
    let mime_path1_user = home_dir()
        .map(|home| home.join(".config/mimeapps.list"))
        .unwrap();
    let mime_path2_user = home_dir()
        .map(|home| home.join(".local/share/applications/mimeapps.list"))
        .unwrap();
    let mime_path_sys = PathBuf::from("/usr/share/applications/mimeinfo.cache");

    mime_paths.push(mime_path1_user);
    mime_paths.push(mime_path2_user);
    mime_paths.push(mime_path_sys);

    let mime_paths: Vec<PathBuf> = mime_paths
        .into_iter()
        .filter(|path| path.exists())
        .collect();
    println!("Avilable mime files: {:?}", mime_paths);

    let app = RelmApp::new("xdg-mimer");
    app.run::<AppModel>(mime_paths);
}
