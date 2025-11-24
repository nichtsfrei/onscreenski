use gdk4::prelude::ObjectExt;
use gtk::prelude::{ApplicationExt, BoxExt, GtkWindowExt, WidgetExt};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::{
    ComponentParts, ComponentSender, RelmRemoveAllExt, SimpleComponent,
    gtk::{self, Label},
};

use crate::{
    ProgramArgs,
    layout::{self, parse::LayoutDefinition},
    service::IPCHandle,
    ui::supported_keys::SupportedKeys,
};

use super::components::ButtonEX;

pub struct UIModel {
    keyboard_handle: Box<dyn IPCHandle>,
    keyboard_definition: LayoutDefinition,
    key_height: i32,
    switch: u8,
    toggled: [bool; 255],
}

#[derive(Debug)]
pub enum UIMessage {
    OneShot(SupportedKeys),
    Layer(u8),
    AppQuit,
}

impl UIModel {
    fn draw_layer(
        sender: ComponentSender<Self>,
        widget: &(gtk::Box, gtk::Box),
        toggled: &[bool],
        key_height: i32,
        layer: &layout::parse::Layer,
        layer_no: u8,
    ) {
        let (left_layout, right_layout) = layer;
        let (left_container, right_container) = widget;

        Self::draw_layout(
            sender.clone(),
            left_container,
            toggled,
            gtk::Align::Start,
            key_height,
            left_layout,
            format!("layer{layer_no}"),
        );
        Self::draw_layout(
            sender,
            right_container,
            toggled,
            gtk::Align::End,
            key_height,
            right_layout,
            format!("layer{layer_no}"),
        );
    }

    fn draw_layout(
        sender: ComponentSender<Self>,
        container: &gtk::Box,
        toggled: &[bool],
        halign: gtk::Align,
        button_height: i32,
        layout: &layout::Layout,
        css_class: String,
    ) {
        container.remove_all();
        layout.iter().for_each(|row| {
            let row_container = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .halign(halign)
                .build();

            row.iter().for_each(|key| {
                let scan_code = key.scan_code;
                let width =
                    (key.width.unwrap_or(1.0) * f32::from(button_height as u16)).round() as i32;

                if scan_code == SupportedKeys::KC_NO {
                    let label = Label::default();
                    label.set_width_request(width);
                    label.add_css_class(&css_class);
                    row_container.append(&label);
                } else {
                    let button = ButtonEX::default();

                    button.set_primary_content(key.top_legend.clone().unwrap_or_default());
                    button.set_secondary_content(key.bottom_legend.clone().unwrap_or_default());

                    button.set_width_request(width);
                    button.set_height_request(button_height);

                    let release_sender = sender.clone();

                    button.connect("released", true, move |_| {
                        release_sender.input(UIMessage::OneShot(scan_code));
                        None
                    });

                    button.add_css_class(&css_class);
                    if toggled[scan_code.as_key_code() as usize] {
                        button.add_css_class("toggled");
                    }
                    row_container.append(&button);
                }
            });

            container.append(&row_container);
        });
    }
}

impl SimpleComponent for UIModel {
    type Init = (Box<dyn IPCHandle>, LayoutDefinition, ProgramArgs);

    type Input = UIMessage;
    type Output = ();
    type Root = gtk::Window;
    type Widgets = (gtk::Box, gtk::Box);

    fn init_root() -> Self::Root {
        // Create a window with a height of 1/3 of the smallest monitor.
        gtk::Window::builder().build()
    }

    // Initialize the UI.
    fn init(
        handle: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Create a thread to listen for the close command.
        window.init_layer_shell();
        window.set_namespace(Some("onscreenski"));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);

        let anchors = [
            (Edge::Left, true),
            (Edge::Right, true),
            (Edge::Top, false),
            (Edge::Bottom, true),
        ];

        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }

        let model = UIModel {
            keyboard_handle: handle.0,
            keyboard_definition: handle.1,
            key_height: handle.2.height,
            switch: 0,
            toggled: [false; 255],
        };

        // window.emit_enable_debugging(true);

        let main_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Fill)
            .build();

        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Start)
            .vexpand(true)
            .build();
        let container2 = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::End)
            .vexpand(true)
            .build();
        //container.set_margin_all(5);
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        main_container.append(&container);
        main_container.append(&spacer);
        main_container.append(&container2);

        let widgets = (container, container2);
        let root_layout = model.keyboard_definition.layer.first().unwrap();
        Self::draw_layer(
            sender,
            &widgets,
            &model.toggled,
            model.key_height,
            root_layout,
            0,
        );
        window.set_child(Some(&main_container));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            UIMessage::AppQuit => {
                for i in 0..254 {
                    if self.toggled[i] {
                        self.keyboard_handle.send(&[i as u8]);
                    }
                }
                //self.keyboard_handle.destroy();
                relm4::main_application().quit();
            }
            UIMessage::OneShot(kc) => {
                if let SupportedKeys::LAYER(layer) = kc {
                    self.update(UIMessage::Layer(layer), _sender);
                } else if kc == SupportedKeys::CLOSE {
                    self.update(UIMessage::AppQuit, _sender);
                } else {
                    if kc.is_mod_key() | kc.is_lock_key() {
                        self.toggled[kc.as_key_code() as usize] =
                            !self.toggled[kc.as_key_code() as usize];
                    } else {
                        // could be expensive?
                        for i in 0..254 {
                            if self.toggled[i] {
                                self.toggled[i] = !self.toggled[i];
                            }
                        }
                    }

                    self.keyboard_handle.send(&[kc.as_key_code()])
                }
            }

            UIMessage::Layer(n) => self.switch = n,
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        let n = self.switch;
        if n >= self.keyboard_definition.layer.len() as u8 {
            panic!(
                "Trying to switch to an unknown layer {n} of {}",
                self.keyboard_definition.layer.len()
            );
        }
        let layer = self.keyboard_definition.layer.get(n as usize).unwrap();
        Self::draw_layer(
            sender.clone(),
            widgets,
            &self.toggled,
            self.key_height,
            layer,
            n,
        );
    }
}
