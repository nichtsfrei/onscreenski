use relm4::RelmApp;
use tracing::info;

use crate::{
    ProgramArgs,
    layout::parse::LayoutDefinition,
    ui::{UIMessage, UIModel},
};

use super::IPCHandle;

pub struct AppService<M: IPCHandle + 'static> {
    ui_handle: RelmApp<UIMessage>,
    keyboard_handle: M,
    layout_definition: LayoutDefinition,
    args: ProgramArgs,
}

impl<M: IPCHandle + 'static> AppService<M> {
    pub fn new(
        keyboard_handle: M,
        layout_definition: LayoutDefinition,
        styles: String,
        args: ProgramArgs,
    ) -> Self {
        let ui = RelmApp::new("com.github.nichtsfrei.onscreenski");

        relm4::set_global_css(&styles);

        Self {
            ui_handle: ui,
            keyboard_handle,
            layout_definition,
            args,
        }
    }

    pub fn run(self) {
        info!("Starting UI.");
        self.ui_handle.with_args(vec![]).run::<UIModel>((
            Box::new(self.keyboard_handle),
            self.layout_definition,
            self.args,
        ));
    }
}
