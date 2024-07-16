mod sound;

use clap::Parser;
use rodio;
use rodio::{OutputStream, Sink};
use sound::{hi_hat_hi, hi_hat_low, Sound};
use std::io::stdin;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Starting tempo (20-500)
    #[arg(short, long, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    start: u16,

    /// End goal tempo (20-500)
    #[arg(short, long, default_value_t = 180, value_parser = clap::value_parser!(u16).range(20..501))]
    end: u16,

    /// Increase step
    #[arg(short, long, default_value_t = 2)]
    increase: u8,

    /// Decrease step
    #[arg(short, long, default_value_t = 1)]
    decrease: u8,

    /// Beats per bar
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..))]
    beats_per_bar: u8,

    /// Length of play segment in bars
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..))]
    length: u8,
}

fn main() {
    let args = Args::parse();

    let tempo: i64 = args.start as i64;
    let end_tempo: i64 = args.end as i64;
    let increase: i64 = args.increase as i64;
    let decrease: i64 = args.decrease as i64;
    let beats_per_bar: i64 = args.beats_per_bar as i64;
    let bars: i64 = args.length as i64;

    let (tx, rx) = mpsc::channel::<bool>();

    thread::spawn(move || {
        input_handler(tx);
    });

    metronome_b(
        tempo,
        end_tempo,
        increase,
        decrease,
        beats_per_bar,
        bars,
        rx,
    );
}

fn input_handler(tx: Sender<bool>) {
    loop {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                if input.starts_with("q") {
                    tx.send(true).unwrap();
                    break;
                } else {
                    tx.send(false).unwrap();
                }
            }
            Err(e) => {
                println!("error: {e}");
                break;
            }
        }
    }
}

fn metronome_b(
    mut tempo: i64,
    end_tempo: i64,
    increase: i64,
    decrease: i64,
    beats_per_bar: i64,
    mut bars: i64,
    rx: Receiver<bool>,
) {
    let mut segment = beats_per_bar * bars;
    let target_bars_divisor: f64 = (tempo * beats_per_bar) as f64;
    let start_segment: f64 = segment as f64;

    let mut incrementor = [increase; 2];
    if decrease > 0 {
        incrementor[1] = decrease * -1;
    }
    let mut alternator: usize = 0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let sound_hi = Sound::get(hi_hat_hi()).unwrap();
    let sound_low = Sound::get(hi_hat_low()).unwrap();

    let mut start = std::time::Instant::now();
    loop {
        println!("Tempo: {}, Bars: {}", tempo, bars);
        let delay = Duration::from_millis((60000 / tempo) as u64);

        for n in 0..segment {
            thread::sleep(delay.saturating_sub(std::time::Instant::now() - start));
            start = std::time::Instant::now();
            if n % beats_per_bar == 0 {
                sink.append(sound_low.decoder());
            } else {
                sink.append(sound_hi.decoder());
            }
            sink.sleep_until_end();

            match rx.try_recv() {
                Ok(r) if r => return,
                Ok(_) => break,
                Err(_) => (),
            }
        }
        if tempo >= end_tempo {
            break;
        }
        tempo += incrementor[alternator];
        bars = f64::round(start_segment * tempo as f64 / target_bars_divisor) as i64;
        segment = beats_per_bar * bars;
        alternator = alternator ^ 1;
    }
}
