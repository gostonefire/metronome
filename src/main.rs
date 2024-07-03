mod sound;

use std::time::Duration;
use rodio;
use rodio::{OutputStream, Sink};
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

fn main() {
    let args = Args::parse();

    let tempo: f64 = args.start as f64;
    let end_tempo: f64 = args.end as f64;
    let increase: f64 = args.increase as f64;
    let decrease: f64 = args.decrease as f64 * -1.0;
    let segment: i64 = args.length as i64;

    metronome_b(tempo, end_tempo, increase, decrease, segment);

}

fn metronome_b(mut tempo: f64, end_tempo: f64, increase: f64, decrease: f64, segment: i64) {
    let mut incrementor: f64 = decrease;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let sound = Sound::get().unwrap();

    let mut start = std::time::Instant::now();
    loop {
        println!("Tempo: {}", tempo);
        let delay = Duration::from_millis((60000.0 / tempo) as u64);

        for _ in 0..segment {
            std::thread::sleep(delay - (std::time::Instant::now() - start));
            start = std::time::Instant::now();
            sink.append(sound.decoder());
            sink.sleep_until_end();
        }
        if tempo >= end_tempo {
            break;
        }
        incrementor = if incrementor < 0.0 {increase} else {decrease};
        tempo += incrementor;
    }
}
