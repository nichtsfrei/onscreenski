pub mod host;

pub trait IPCHandle {
    fn send(&self, data: &[u8]);
}
