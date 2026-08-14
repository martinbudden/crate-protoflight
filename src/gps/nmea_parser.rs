#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum NmeaParser {
    #[default]
    Init,
}

impl NmeaParser {
    pub const fn new() -> Self {
        Self::Init
    }
    // TODO: placeholder
    pub fn on_data_received(&mut self, _data: u8) -> bool {
        _ = self;
        true
    }
}
