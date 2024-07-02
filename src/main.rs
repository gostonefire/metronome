mod sound;

use std::time::Duration;
use rodio;
use rodio::{OutputStream, Sink};
use tokio::time::Instant;
use sound::Sound;

#[tokio::main]
async fn main() {
    let mut tempo: f64 = 60.0;
    let segment: i64 = 5;
    let mut incrementor: f64 = -1.0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let sound = Sound::load("tracks/UHH_AcoustiHatB_Hit-02_trunc.wav").unwrap();


    let mut start = Instant::now();
    loop {
        println!("Tempo: {}", tempo);
        let delay = Duration::from_millis((60000.0 / tempo) as u64);
        let mut interval = tokio::time::interval_at(start + delay, delay);

        for n in 0..segment {
            tokio::select! {
                _ = interval.tick() => {
                    start = Instant::now();
                    sink.append(sound.decoder());
                    sink.sleep_until_end();
                }
            }
        }
        incrementor = if incrementor < 0.0 {2.0} else {-1.0};
        tempo += incrementor;
    }
}
