mod sound;

use clap::Parser;
use rodio;
use rodio::{OutputStream, Sink};
use sound::{kick, kick_hi_hat, hi_hat_hi, Sound};
use std::str::FromStr;
use std::{fmt, io, thread};
use std::io::Write;
use std::time::Duration;

#[derive(Debug, Clone)]
struct DecodeError(String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Composition decode error: {}", self.0)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Starting tempo (20-500)
    #[arg(short, long, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    start: u16,

    /// End goal tempo (20-500)
    #[arg(short, long, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    end: u16,

    /// Increase step
    #[arg(short, long, default_value_t = 0)]
    increase: u8,

    /// Decrease step
    #[arg(short, long, default_value_t = 0)]
    decrease: u8,

    /// Composition of bar
    #[arg(short, long, default_value = "4k 4h 4h 4h")]
    composition: String,

    /// Length of play segment in bars
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    length: u8,
}

fn main() {
    let args = Args::parse();

    let tempo: i64 = args.start as i64;
    let end_tempo: i64 = if args.end > args.start {args.end as i64} else {tempo};
    let increase: i64 = args.increase as i64;
    let decrease: i64 = args.decrease as i64;
    let composition: String = args.composition;

    let bars: i64 = args.length as i64;

    let bar = match decode(composition) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    metronome(
        tempo,
        end_tempo,
        increase,
        decrease,
        bar,
        bars,
    );
}

fn decode(composition: String) -> Result<Vec<(f64, usize, char)>, DecodeError> {
    let mut bar: Vec<(f64, usize, char)> = Vec::new();

    for p in composition
        .trim()
        .split(' ')
        .map(|x| format!("{:0>3}", x.trim()))
        .collect::<Vec<String>>() {
        if p.len() != 3 {
            return Err(DecodeError("malformed beat definition"
                .to_string()));
        }

        let n = u8::from_str(&p[0..2])
            .map_err(|_| DecodeError("illegal note found, should be one of 1, 2, 4, 8, 16"
                .to_string()))?;
        let t: f64 = match n {
            1  => 16f64,
            2  => 8f64,
            4  => 4f64,
            8  => 2f64,
            16 => 1f64,
            _  => return Err(DecodeError("illegal note found, should be one of 1, 2, 4, 8, 16"
                .to_string())),
        };
        let i: usize;
        let c: char;
        match &p[2..3] {
            "k" => {i = 0; c = '\0';},
            "m" => {i = 1; c = '*';},
            "h" => {i = 2; c = '*';},
            "p" => {i = 3; c = '\0';},
            _   => return Err(DecodeError("illegal sound found, should be one of k, m, h, p"
                .to_string())),
        };

        bar.push((t, i, c));
    }

    if bar.len() == 0 {
        Err(DecodeError("empty composition"
            .to_string()))
    } else {
        Ok(bar)
    }
}

fn metronome(
    mut tempo: i64,
    end_tempo: i64,
    increase: i64,
    decrease: i64,
    bar: Vec<(f64,usize, char)>,
    mut bars: i64,
) {
    let start_tempo = tempo as f64;
    let start_bars = bars as f64;

    let mut incrementor = [increase; 2];
    if decrease > 0 {
        incrementor[1] = decrease * -1;
    }
    let mut alternator: usize = 0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let sound: [Sound;3] = [
        Sound::get(kick()).unwrap(),
        Sound::get(kick_hi_hat()).unwrap(),
        Sound::get(hi_hat_hi()).unwrap(),
    ];

    println!("Tempo: {}, Bars: {}", tempo, bars);

    let mut start = std::time::Instant::now();
    let mut note: f64 = 0f64;
    loop {

        let sixteenth: f64 = (60000f64 / tempo as f64) / 4f64;

        for _ in 0..bars {
            for n in &bar {
                let delay = Duration::from_millis((note * sixteenth) as u64);
                thread::sleep(delay.saturating_sub(std::time::Instant::now() - start));
                start = std::time::Instant::now();
                if n.1 < 3 {
                    print!("{}", n.2);
                    io::stdout().flush().unwrap();
                    sink.append(sound[n.1].decoder());
                    sink.sleep_until_end();
                }
                note = n.0;
            }
            println!();
        }

        tempo += incrementor[alternator];
        bars = f64::round(tempo as f64 / start_tempo * start_bars) as i64;
        if tempo > end_tempo {
            break;
        }

        if incrementor[alternator] != 0 {
            println!("Tempo: {}, Bars: {}", tempo, bars);
        }
        alternator = alternator ^ 1;
    }
}
