mod sound;

use std::io::stdin;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
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
    let decrease: f64 = args.decrease as f64;
    let segment: i64 = args.length as i64;

    let (tx, rx) = mpsc::channel::<bool>();

    thread::spawn(move || {
        input_handler(tx);
    });

    metronome_b(tempo, end_tempo, increase, decrease, segment, rx);
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

            },
            Err(e) => {
                println!("error: {e}");
                break;
            },
        }
    }
}

fn metronome_b(mut tempo: f64, end_tempo: f64, increase: f64, decrease: f64, segment: i64, rx: Receiver<bool>) {
    let mut incrementor = [increase;2];
    if decrease > 0.0 {
        incrementor[1] = decrease * -1.0;
    }
    let mut alternator: usize = 0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let sound = Sound::get().unwrap();

    let mut start = std::time::Instant::now();
    loop {
        println!("Tempo: {}", tempo);
        let delay = Duration::from_millis((60000.0 / tempo) as u64);

        for _ in 0..segment {
            thread::sleep(delay.saturating_sub(std::time::Instant::now() - start));
            start = std::time::Instant::now();
            sink.append(sound.decoder());
            sink.sleep_until_end();

            match rx.try_recv() {
                Ok(r) if r => return,
                Ok(_) => break,
                Err(_) => ()
            }
        }
        if tempo >= end_tempo {
            break;
        }
        tempo += incrementor[alternator];
        alternator = alternator ^ 1;
    }
}
