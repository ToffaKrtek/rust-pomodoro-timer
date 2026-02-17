use gtk::prelude::*;
use notify_rust::Notification;
use std::{cell::RefCell, path::Path, process::Command, rc::Rc};

use glib::{self, ControlFlow};
use gtk::{Box as GtkBox, Button, Label, Orientation, Window};

fn play_sound(sound_type: &str) {
    let sound_path = match sound_type {
        "work" => "/usr/share/sounds/freedesktop/stereo/complete.oga",
        "break" => "/usr/share/sounds/freedesktop/stereo/message.oga",
        _ => "/usr/share/sounds/freedesktop/stereo/bell.oga",
    };
    if Path::new(sound_path).exists() {
        let _ = Command::new("paplay").arg(sound_path).spawn();
    } else {
        let _ = Command::new("beep").spawn();
    }
}

const WORK_TIME: u32 = 15 * 60;
const BREAK_TIME: u32 = 5 * 60;

struct TimerState {
    time_left: u32,
    running: bool,
    work_mode: bool,
}

impl TimerState {
    fn new() -> Self {
        Self {
            time_left: WORK_TIME,
            running: false,
            work_mode: true,
        }
    }

    fn toggle_mode(&mut self) {
        self.work_mode = !self.work_mode;
        self.reset();
    }
    fn reset(&mut self) {
        self.time_left = if self.work_mode {
            WORK_TIME
        } else {
            BREAK_TIME
        }
    }
    fn tick(&mut self) {
        if self.running && self.time_left > 0 {
            self.time_left -= 1;
        }
    }
}

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to init GTK.");
        return;
    }
    let window = Window::new(gtk::WindowType::Toplevel);
    window.set_title("Pomodoro Timer");
    window.set_default_size(300, 200);
    window.set_position(gtk::WindowPosition::Center);

    let vbox = GtkBox::new(Orientation::Vertical, 5);
    window.add(&vbox);

    let label = Label::new(None);
    label.set_markup("<span font='48' weight='bold'>15:00</span>");
    vbox.pack_start(&label, true, true, 10);

    let start_button = Button::with_label("Старт");
    vbox.pack_start(&start_button, false, false, 5);

    let reset_button = Button::with_label("Сброс");
    vbox.pack_start(&reset_button, false, false, 5);

    let state = Rc::new(RefCell::new(TimerState::new()));

    let update_label = {
        let label = label.clone();
        let state = state.clone();

        move || {
            let state = state.borrow();
            let minutes = state.time_left / 60;
            let seconds = state.time_left % 60;
            label.set_markup(&format!(
                "<span font='48' weight='bold'>{:02}:{:02}</span>",
                minutes, seconds
            ));
        }
    };
    update_label();
    {
        let state = state.clone();
        // let start_button = start_button.clone();
        start_button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.running = !state.running;
            // let label_text = if state.running {
            //     "Пауза"
            // } else {
            //     "Старт"
            // };
            // start_button.set_label(label_text);
        });
    }
    {
        let state = state.clone();
        let update_label = update_label.clone();
        reset_button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.reset();
            drop(state);
            update_label();
        });
    }
    let update_label_timer = update_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        let mut state = state.borrow_mut();
        if state.running {
            state.tick();
            if state.time_left == 0 {
                state.toggle_mode();
                let sound_type = if state.work_mode { "work" } else { "break" };
                play_sound(sound_type);
                let summary = if state.work_mode {
                    "🍅 Работа"
                } else {
                    "☕ Отдых"
                };
                let body = if state.work_mode {
                    "Время поработать"
                } else {
                    "Пора сделать перерыв"
                };
                let _ = Notification::new().summary(summary).body(body).show();
            }
        }
        drop(state);
        update_label_timer();
        ControlFlow::Continue
    });
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        glib::Propagation::Proceed
    });
    window.show_all();
    gtk::main();
}
