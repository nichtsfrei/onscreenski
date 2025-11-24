mod config;
mod layout;
mod service;
mod ui;
mod utils;

use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Mutex;

use clap::Parser;
use config::AppConfig;
use layout::parse::LayoutDefinition;
use service::host::AppService;
use tracing::info;
use utils::ProgramArgs;

struct MayI {
    socket: Mutex<UnixStream>,
}

impl Default for MayI {
    fn default() -> Self {
        let socket = Mutex::new(UnixStream::connect(socket_path()).unwrap());
        Self { socket }
    }
}

fn socket_path() -> PathBuf {
    const SOCKET_NAME: &str = "ukeynski.socket";
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR").unwrap_or_else(|| "/tmp".into());

    let mut path = PathBuf::from(runtime_dir);
    path.push(SOCKET_NAME);

    path
}

// Can actually be removed
impl service::IPCHandle for MayI {
    fn send(&self, data: &[u8]) {
        let mut socket = self.socket.lock().unwrap();
        socket.write_all(data).unwrap();
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = ProgramArgs::parse();
    info!("Message: {:?}", args);

    info!("Starting app service.");

    ctrlc::set_handler(move || {
        info!("Received SIGINT. Exiting.");
        std::process::exit(0);
    })
    .unwrap();

    let app_config = AppConfig::new(args.layout.clone(), args.style.clone());
    let user_layout = LayoutDefinition::from_toml(&app_config.get_layout_file_content());
    let user_style = app_config.get_css_file_content();

    let keyboard = MayI::default();

    AppService::new(keyboard, user_layout, user_style, args).run();
    info!("App Service Exiting.");

    info!("Exited");
}
