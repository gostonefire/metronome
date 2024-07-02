mod sound;

use std::time::Duration;
use rodio;
use rodio::{OutputStream, Sink};
use tokio::time::Instant;
use sound::Sound;
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Starting tempo
    #[arg(short, long, default_value_t = 60)]
    start: u8,

    /// End goal tempo
    #[arg(short, long, default_value_t = 180)]
    end: u8,

    /// Increase step
    #[arg(short, long, default_value_t = 2)]
    increase: u8,

    /// Decrease step
    #[arg(short, long, default_value_t = 1)]
    decrease: u8,

    /// Length of play segment
    #[arg(short, long, default_value_t = 10)]
    length: u8,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut tempo: f64 = args.start as f64;
    let end_tempo: f64 = args.end as f64;
    let increase: f64 = args.increase as f64;
    let decrease: f64 = args.decrease as f64 * -1.0;
    let segment: i64 = args.length as i64;

    let mut incrementor: f64 = decrease;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let sound = Sound::load("tracks/UHH_AcoustiHatB_Hit-02_trunc.wav").unwrap();


    let mut start = Instant::now();
    loop {
        println!("Tempo: {}", tempo);
        let delay = Duration::from_millis((60000.0 / tempo) as u64);
        let mut interval = tokio::time::interval_at(start + delay, delay);

        for _ in 0..segment {
            tokio::select! {
                _ = interval.tick() => {
                    start = Instant::now();
                    sink.append(sound.decoder());
                    sink.sleep_until_end();
                }
            }
        }
        if tempo >= end_tempo {
            break;
        }
        incrementor = if incrementor < 0.0 {increase} else {decrease};
        tempo += incrementor;
    }
}
