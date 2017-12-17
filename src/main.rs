extern crate gtk;
extern crate gdk;
extern crate regex;
extern crate sourceview;

use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::string::String;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::vec::Vec;
use std::fs;
use std::sync::Arc;
use regex::Regex;
use sourceview::LanguageManagerExt;

struct Gui {
    // right_column: gtk::Paned,
    headerbar     : gtk::HeaderBar,
    file_tree     : gtk::TreeView,
    file_scroll   : gtk::ScrolledWindow,
    edit_view     : sourceview::View,
    edit_scroll   : gtk::ScrolledWindow,
    result_view   : gtk::TextView,
    result_scroll : gtk::ScrolledWindow,
    run_button    : RefCell<gtk::Button>,
    save_button   : RefCell<gtk::Button>,
    filename      : RefCell<String>,
}

impl Gui {
    fn new() -> Gui {
        let lm = sourceview::LanguageManager::new();
        let lang = lm.get_language("python").unwrap();
        return Gui {
            // right_column: ,
            headerbar    : gtk::HeaderBar::new(),
            file_tree    : gtk::TreeView::new(),
            file_scroll  : gtk::ScrolledWindow::new(None, None),
            edit_view    : sourceview::View::new_with_buffer(&sourceview::Buffer::new_with_language(&lang)),
            edit_scroll  : gtk::ScrolledWindow::new(None,None),
            result_view  : gtk::TextView::new(),
            result_scroll: gtk::ScrolledWindow::new(None,None),
            run_button   : RefCell::new(gtk::Button::new()),
            save_button  : RefCell::new(gtk::Button::new()),
            filename     : RefCell::new(String::from("src/main.py")),
        };
    }

    fn init(&mut self) {
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
                .arg("/C")
                .arg("@cd")
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

    fn get_text_of_file(filename: &String) -> String{
        let mut ret_string: String = String::new();

        match File::open(filename) {
            Ok(mut file) => { let _ = file.read_to_string(&mut ret_string); }
            Err(why) => { panic!(why.to_string()) }
        }

        return ret_string;
    }

    fn set_save_button(&mut self) {
        self.save_button.get_mut().set_label("Save");
        struct SaveButton {
            textbuffer : gtk::TextBuffer,
        }
        struct SaveButtonWrap {
            refs : RefCell<SaveButton>,
        }

        let buf = self.edit_view.get_buffer().unwrap();
        let save_button_ref = Rc::new(SaveButtonWrap {
            refs : RefCell::new(SaveButton {
                textbuffer : buf,
            })
        });
        {
            let mut_filename= &mut self.filename as *mut RefCell<String>;
            let refs_tmp = save_button_ref.clone();
            self.save_button.get_mut().connect_clicked(move |_| {
                let refs = refs_tmp.refs.borrow();
                let mut filename_clone = mut_filename.clone();
                unsafe {
                    match File::create(&format!("./{}", (*filename_clone).get_mut())) {
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
                }
            });
        }
    }

    fn set_run_button(&mut self) {
        self.run_button.get_mut().set_label("Run");

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
            let mut_filename= &mut self.filename as *mut RefCell<String>;
            let refs_tmp = run_button_ref.clone();
            self.run_button.get_mut().connect_clicked(move |_| {
                let mut filename_clone = mut_filename.clone();
                let refs = refs_tmp.refs.borrow();

                unsafe {
                    let output = if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args(&["/C", &format!("{}/bin/Python36/python {}/{}", Gui::get_pwd(), Gui::get_pwd(), *(*filename_clone).borrow())])
                            .output()
                            .expect("failed to execute process")
                    } else {
                        Command::new("sh")
                            .arg("-c")
                            .arg(format!("python {}/{}", Gui::get_pwd(), *(*filename_clone).borrow()))
                            .output()
                            .expect("failed to execute process")
                    };
                    let cmd_out = String::from_utf8_lossy(&output.stdout);
                    let cmd_err = String::from_utf8_lossy(&output.stderr);
                    refs.textbuffer.set_text(
                        &format!("{}{}", &cmd_out, &cmd_err)
                        );
                }
            });
        }
    }

    pub fn set_header(&mut self) {
        unsafe{
            self.headerbar.pack_start(&(*self.run_button.get_mut()));
            self.headerbar.pack_start(&(*self.save_button.get_mut()));
            self.headerbar.set_show_close_button(true);
            // return self.headerbar;
        }
    }

    fn get_new_column(title: &str, column_num: i32) -> gtk::TreeViewColumn {
        let column   = gtk::TreeViewColumn::new();
        column.set_title(title);

        let cell = gtk::CellRendererText::new();

        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", column_num as i32);

        return column;
    }

    fn get_file_name(filepath: &String) -> String  {
        let re = Regex::new (r"/|\\").unwrap();
        // let mut splited: Vec<&str> = filepath.split("/").collect();
        let mut splited: Vec<&str> = re.split(filepath).collect();
        return splited.pop().unwrap().to_string() as String;
    }

    fn set_file_tree_store(dir: String, iter: Option<&gtk::TreeIter>, store: &gtk::TreeStore) {
        let dir_iter = match iter {
            Some(iter_tmp) => { store.insert_with_values(Some(&iter_tmp), None, &[0], &[&Gui::get_file_name(&dir)]) },
            None => { store.insert_with_values(None, None, &[0], &[&Gui::get_file_name(&dir)]) },
        };
        if let Ok(paths) =  fs::read_dir(dir) {
            for path in paths {
                let pathbuf = path.unwrap().path();
                let filename = String::from(pathbuf.to_str().unwrap());
                if !pathbuf.is_file() {
                    Gui::set_file_tree_store(filename.clone(), Some(&dir_iter), &store);
                }else{
                    let _ = store.insert_with_values(Some(&dir_iter), None, &[0], &[&Gui::get_file_name(&filename)]);
                }
            }
        }
    }

    fn get_fullpath(treemodel: gtk::TreeModel, treepath: &mut gtk::TreePath) -> String {
        let iter  = treemodel.get_iter(&treepath).unwrap();
        let value = treemodel.get_value(&iter, 0).get::<String>().unwrap();
        treepath.up();
        if treepath.get_depth() > 0 {
            let downpath = Gui::get_fullpath(treemodel, treepath);
            return format!("{}/{}", downpath, value);
        }
        return value;
    }

    // pub fn set_file_tree(&self) {
    pub fn set_file_tree(&mut self) {
        let column_types   = [gtk::Type::String];
        let file_store = gtk::TreeStore::new(&column_types);

        let file_column_num  = 0;

        let file_column  = Gui::get_new_column("Files", file_column_num);

        self.file_tree.append_column(&file_column);
        self.file_tree.set_model(Some(&file_store));

        Gui::set_file_tree_store(String::from("./src"), None, &file_store);

        struct FileTree {
            textbuffer : gtk::TextBuffer,
        }
        struct FileTreeWrap{
            refs : RefCell<FileTree>,
        }

        let buf = self.edit_view.get_buffer().unwrap();
        let file_tree_ref = Rc::new(FileTreeWrap{
            refs : RefCell::new(FileTree{
                textbuffer : buf,
            })
        });
        {
            let mut_filename = &mut self.filename as *mut RefCell<String>;
            let refs_tmp = file_tree_ref.clone();
            self.file_tree.connect_row_activated(move |treeview, treepath, treeviewcolumn| {
                let mut filename_clone = mut_filename.clone();
                let refs = refs_tmp.refs.borrow();

                let model = treeview.get_model().unwrap();
                unsafe {
                    *(*filename_clone).get_mut() = format!("{}", Gui::get_fullpath(model, &mut treepath.clone()));
                    // (*filename_clone).get_mut() == 1;
                    refs.textbuffer.set_text(
                        &Gui::get_text_of_file((*filename_clone).get_mut())
                        );
                }
            });
        }
        self.file_scroll.add(&self.file_tree);
    }

    pub fn set_result_view(&self) {
        self.result_view.set_editable(false);
        self.result_scroll.add(&self.result_view);
    }

    pub fn set_edit_view(&mut self) {
        self.edit_view.get_buffer().unwrap().set_text(&Gui::get_text_of_file(&self.filename.borrow().clone()));
        {
            let mut_rbtn = &mut self.run_button  as *mut RefCell<gtk::Button>;
            let mut_sbtn = &mut self.save_button as *mut RefCell<gtk::Button>;
            self.edit_view.connect_key_press_event(move |_,event_key| {
                let mut rbtn_clone = mut_rbtn.clone();
                let mut sbtn_clone = mut_sbtn.clone();

                // ctrl(4)が他のキーと一緒に押されている
                if event_key.get_state().bits() == 4 {
                    unsafe {
                        match event_key.get_keyval() as u32 {
                            114 => { // r button
                                (*rbtn_clone).borrow().clicked();
                            }
                            115 => { // s button
                                (*sbtn_clone).borrow().clicked();
                            }
                            key => { println!("{}", key) }
                        }
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

    let mut gui = Gui::new();
    gui.init();

    let mut main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    let two_column    = gtk::Paned::new(gtk::Orientation::Horizontal);
    let right_column  = gtk::Paned::new(gtk::Orientation::Vertical);

    init(&mut main_window);

    main_window.connect_delete_event(|_,_| {
        gtk::main_quit();
        gtk::prelude::Inhibit(false)
    });

    two_column.pack1(&gui.file_scroll,     true, false);
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

    two_column  .set_position(150);
    right_column.set_position(height-150);

    gtk::main();
}

fn init(main_window: &gtk::Window) {
    main_window.set_title("IDEP");

    main_window.set_border_width(10);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_default_size(1200,700);
}

