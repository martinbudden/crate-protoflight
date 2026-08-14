#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UbloxParser {
    #[default]
    Init,
}

impl UbloxParser {
    pub const fn new() -> Self {
        Self::Init
    }
    // TODO: placeholder
    pub fn on_data_received(&mut self, _data: u8) -> bool {
        _ = self;
        true
    }
}
