extern crate gtk;
extern crate gdk;

use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::string::String;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

struct Gui {
    // right_column: gtk::Paned,
    headerbar     : gtk::HeaderBar,
    file_tree     : gtk::ScrolledWindow,
    edit_view     : gtk::TextView,
    edit_scroll   : gtk::ScrolledWindow,
    result_view   : gtk::TextView,
    result_scroll : gtk::ScrolledWindow,
    run_button    : gtk::Button,
    save_button   : gtk::Button,
    filename      : String,
}

impl Gui {
    fn new() -> Gui {
        return Gui {
            // right_column: ,
            headerbar    : gtk::HeaderBar::new(),
            file_tree    : gtk::ScrolledWindow::new(None, None),
            edit_view    : gtk::TextView::new(),
            edit_scroll  : gtk::ScrolledWindow::new(None,None),
            result_view  : gtk::TextView::new(),
            result_scroll: gtk::ScrolledWindow::new(None,None),
            run_button   : gtk::Button::new(),
            save_button  : gtk::Button::new(),
            filename     : String::from("./src/main.py"),
        };
    }
    fn init(&self) {
        self.set_run_button();
        self.set_save_button();
        self.set_header();
        self.set_file_tree();
        self.set_edit_view();
        self.set_result_view();
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

    fn get_text_of_file(&self) -> String{
        let mut ret_string: String = String::new();

        match File::open(&self.filename) {
            Ok(mut file) => { let _ = file.read_to_string(&mut ret_string); }
            Err(why) => { panic!(why.to_string()) }
        }

        return ret_string;
    }
    fn set_save_button(&self) {
        self.save_button.set_label("Save");

        struct SaveButton {
            textbuffer : gtk::TextBuffer,
            filename   : std::string::String,
        }
        struct SaveButtonWrap {
            refs : RefCell<SaveButton>,
        }

        let buf = self.edit_view.get_buffer().unwrap();
        let fil = self.filename.clone();
        let save_button_ref = Rc::new(SaveButtonWrap {
            refs : RefCell::new(SaveButton {
                textbuffer : buf,
                filename   : fil,
            })
        });
        {
            let refs_tmp = save_button_ref.clone();
            self.save_button.connect_clicked(move |_| {
                let refs = refs_tmp.refs.borrow();
                match File::create(&refs.filename) {
                    Ok(mut file) => {
                        let _ = file.write_all(refs.textbuffer.get_text(
                                &refs.textbuffer.get_start_iter(),
                                &refs.textbuffer.get_end_iter(),
                                false)
                            .unwrap()
                            .as_bytes());
                    }
                    Err(why)     => { panic!(why.to_string()) }
                }
            });
        }
    }

    fn set_run_button(&self) {
        self.run_button.set_label("Run");

        struct RunButton {
            textbuffer : gtk::TextBuffer,
        }
        struct RunButtonWrap {
            refs : RefCell<RunButton>,
        }

        let buf = self.result_view.get_buffer().unwrap();
        let run_button_ref = Rc::new(RunButtonWrap {
            refs : RefCell::new(RunButton {
                textbuffer : buf,
            })
        });
        {
            let refs_tmp = run_button_ref.clone();
            self.run_button.connect_clicked(move |_| {
                let refs = refs_tmp.refs.borrow();
                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(&["/C", &format!("bin/Python36/python {}\\src\\main.py", Gui::get_pwd())])
                        .output()
                        .expect("failed to execute process")
                } else {
                    Command::new("sh")
                        .arg("-c")
                        .arg(format!("python {}/src/main.py", Gui::get_pwd()))
                        .output()
                        .expect("failed to execute process")
                };

                let cmd_out = String::from_utf8_lossy(&output.stdout);
                let cmd_err = String::from_utf8_lossy(&output.stderr);
                // println!("cmd: {}", format!("python {}/src/main.py", Gui::get_pwd()));
                refs.textbuffer.set_text(
                    &format!("{}{}", &cmd_out, &cmd_err)
                    );
            });
        }
    }

    pub fn set_header(&self) {
        self.headerbar.pack_start(&self.run_button);
        self.headerbar.pack_start(&self.save_button);
        // return self.headerbar;
    }

    pub fn set_file_tree(&self) {
        // self.file_tree = gtk::ScrolledWindow::new(None, None);
        // return self.file_tree
    }

    pub fn set_result_view(&self) {
        self.result_view.set_editable(false);
        self.result_scroll.add(&self.result_view);
    }

    pub fn set_edit_view(&self) {
        self.edit_view.get_buffer().unwrap().set_text(&self.get_text_of_file());
        struct EditView {
            run_button  : gtk::Button,
            save_button : gtk::Button,
        }
        struct EditViewWrap {
            refs : RefCell<EditView>,
        }

        let edit_view_ref = Rc::new(EditViewWrap {
            refs : RefCell::new(EditView {
                run_button  : self.run_button.clone(),
                save_button : self.save_button.clone(),
            })
        });
        {
            let refs_tmp = edit_view_ref.clone();
            self.edit_view.connect_key_press_event(move |_,event_key| {
                let refs = refs_tmp.refs.borrow();
                // ctrl(4)が他のキーと一緒に押されている
                if event_key.get_state().bits() == 4 {
                    match event_key.get_keyval() as u32 {
                        114 => { // r button
                            refs.run_button.clicked();
                        }
                        115 => { // s button
                            refs.save_button.clicked();
                        }
                        key => { println!("{}", key) }
                    }
                }
                return gtk::prelude::Inhibit(false);
            });
        }
        self.edit_scroll.add(&self.edit_view);
    }
}

fn main() {
    gtk::init()
        .expect("Failed to initialize GTK");

    let gui = Gui::new();
    gui.init();

    let mut main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    let two_column    = gtk::Paned::new(gtk::Orientation::Horizontal);
    let right_column  = gtk::Paned::new(gtk::Orientation::Vertical);

    init(&mut main_window);

    main_window.connect_delete_event(|_,_| {
        gtk::main_quit();
        gtk::prelude::Inhibit(false)
    });

    two_column.pack1(&gui.file_tree,     true, false);
    two_column.pack2(&right_column,        true, false);

    right_column.pack1(&gui.edit_scroll,   true, false);
    right_column.pack2(&gui.result_scroll, true, false);

    main_window.set_titlebar(Some(&gui.headerbar));
    main_window.add(&two_column);

    main_window.show_all();

    let allocation = two_column.get_allocation();
    let width = allocation.width;
    let height= allocation.height;

    println!("width: {} height: {}", width, height);

    two_column  .set_position(45);
    right_column.set_position(height-40);

    gtk::main();
}

fn init(main_window: &gtk::Window) {
    main_window.set_title("IDEP");

    main_window.set_border_width(10);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_default_size(350,70);
}

