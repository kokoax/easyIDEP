extern crate gtk;
extern crate gdk;

use gtk::prelude::*;
use std::string::String;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

struct gui {
    // right_column: gtk::Paned,
    headerbar   : gtk::HeaderBar,
    file_tree   : gtk::ScrolledWindow,
    edit_view   : gtk::ScrolledWindow,
    result_view : gtk::ScrolledWindow
}

impl gui {
    fn new() -> gui {
        return gui {
            // right_column: ,
            headerbar  : gui.get_header(),
            file_tree  : gui.get_file_tree(),
            edit_view  : gui.get_edit_view(),
            result_view: gui.get_result_view(),
        };
    }
    fn get_pwd() -> String {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", "@cd"])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg("pwd")
                .output()
                .expect("failed to execute process")
        };
        let mut cmd_out = String::from(String::from_utf8_lossy(&output.stdout));
        cmd_out.pop();  // drop freshline
        return cmd_out;
    }

    fn get_text_of_file(&self, filename: String) -> String{
        let mut ret_string: String = String::new();

        match File::open(filename) {
            Ok(mut file) => { file.read_to_string(&mut ret_string); }
            Err(why) => { panic!(why.to_string()) }
        }

        return ret_string;
    }

    fn get_run_button(&self) -> gtk::Button {
        let run_button = gtk::Button::new();

        run_button.set_label("Run");

        run_button.connect_clicked(|button| {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(&["/C", &format!("python {}\\src\\main.py", gui::get_pwd())])
                    .output()
                    .expect("failed to execute process")
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("python {}/src/main.py", gui::get_pwd()))
                    .output()
                    .expect("failed to execute process")
            };

            let cmd_out = output.stdout;
            println!("cmd: {}", format!("python {}/src/main.py", gui::get_pwd()));
            println!("output\n{}", String::from_utf8_lossy(&cmd_out));
        });
        return run_button;
    }

    pub fn get_header(&self) -> gtk::HeaderBar {
        let header = gtk::HeaderBar::new();
        header.pack_start(&gui::get_run_button());

        return header;
    }

    pub fn get_file_tree(&self) -> gtk::ScrolledWindow {
        let file_scroll = gtk::ScrolledWindow::new(None, None);
        return file_scroll;
    }

    pub fn get_result_view(&self) -> gtk::ScrolledWindow {
        let result_view   = gtk::TextView::new();
        let result_scroll = gtk::ScrolledWindow::new(None, None);
        result_view.set_editable(false);

        result_scroll.add(&result_view);
        return result_scroll;
    }

    pub fn get_edit_view(&self) -> gtk::ScrolledWindow {
        let edit_view = gtk::TextView::new();
        let edit_scroll = gtk::ScrolledWindow::new(None, None);
        edit_view.get_buffer().unwrap().set_text(&gui::get_text_of_file(String::from("./src/main.py")));

        edit_scroll.add(&edit_view);
        return edit_scroll;
    }

    fn check_filedir(&self) {
    }
}

fn main() {
    gtk::init()
        .expect("Failed to initialize GTK");

    let gui = gui::new();

    let mut main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    let two_column    = gtk::Paned::new(gtk::Orientation::Horizontal);
    let right_column  = gtk::Paned::new(gtk::Orientation::Vertical);

    init(&mut main_window);

    main_window.connect_delete_event(|_,_| {
        gtk::main_quit();
        gtk::prelude::Inhibit(false)
    });

    two_column.pack1(&gui.get_file_tree(),     true, false);
    two_column.pack2(&right_column,        true, false);

    right_column.pack1(&gui.get_edit_view(),   true, false);
    right_column.pack2(&gui.get_result_view(), true, false);

    main_window.set_titlebar(Some(&gui::get_header()));
    main_window.add(&two_column);

    main_window.show_all();

    let allocation = two_column.get_allocation();
    let width = allocation.width;
    let height= allocation.height;

    println!("width: {} height: {}", width, height);

    two_column  .set_position(45);
    right_column.set_position(height-15);

    gtk::main();
}

fn init(main_window: &gtk::Window) {
    main_window.set_title("IDEP");

    main_window.set_border_width(10);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_default_size(350,70);
}

