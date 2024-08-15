use std::io;
use std::sync::Arc;
use rodio::Decoder;

pub struct Sound (Arc<Vec<u8>>);

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sound {
    pub fn get(v: Vec<u8>) -> io::Result<Sound> {
        Ok(Sound(Arc::new(v)))
    }

    pub fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> Decoder<io::Cursor<Sound>> {
        Decoder::new(self.cursor()).unwrap()
    }
}

pub fn kick() -> Vec<u8> {
    let b = include_bytes!("../../tracks/kick_trunc.wav");
    Vec::from(b)
}

pub fn kick_hi_hat() -> Vec<u8> {
    let b = include_bytes!("../../tracks/kick_and_hihat_trunc.wav");
    Vec::from(b)
}

pub fn hi_hat() -> Vec<u8> {
    let b = include_bytes!("../../tracks/hihat_trunc_0_1.wav");
    Vec::from(b)
}
