mod sound;

use clap::Parser;
use rodio;
use rodio::{OutputStream, Sink};
use std::str::FromStr;
use std::{fmt, io, thread};
use std::io::Write;
use std::time::Duration;
use sound::{hi_hat, kick_hi_hat, kick, Sound};

#[derive(Debug, Clone)]
struct DecodeError(String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Composition decode error: {}", self.0)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, verbatim_doc_comment)]
/// Metronome for speed training on the guitar
///
/// Most switches are quite self-explanatory except the -c/--composition.
/// If not given the default will be a 4/4 with 4 quarter notes starting with a kick drum and 3
/// hi-hats. Each beat will display a star (*) when playing.
///
/// The notation for each beat in a bar comprises three values:
/// - Length of the beat note - 1 (whole), 2 (half), 4 (quarter), 8 (eighth) and 16 (sixteenth)
/// - Sound to use - k (kick drum), h (hi-hat), m (combined kick and hi-hat), p (silent pause)
/// - Play indicator - p (prints a star when playing), s (prints nothing when playing)
///
/// Each beat must be separated by a space.
///
/// There is no hard rule that each bar must sum up to a full whole note, but to correspond
/// to a given tempo it should.
///
/// Example:
/// -c "8mp 8hp 8ks 16hp 16hp 8ks 8ps 16mp 16hp 16ps 16hp"
/// Intends for the user to play on the 1, 2, 4, 5, 8, 9 and 11 beat where the rest is
/// supporting beats or pauses.
struct Args {
    /// Starting quarter note tempo (20-500)
    #[arg(short, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    start: u16,

    /// End goal quarter note tempo (20-500)
    #[arg(short, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    end: u16,

    /// Increase step (alternates with a decrease if decrease is greater than zero)
    #[arg(short, default_value_t = 0)]
    increase: u8,

    /// Decrease step (alternates with an increase if increase is greater than zero)
    #[arg(short, default_value_t = 0)]
    decrease: u8,

    /// Sweep from start to end and back in loop
    #[arg(short)]
    wave: bool,

    /// Composition of bar
    #[arg(short, default_value = "4kp 4hp 4hp 4hp")]
    composition: String,

    /// Length of play segment in bars
    #[arg(short, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    length: u8,
}

fn main() {
    let args = Args::parse();

    let tempo: i64 = args.start as i64;
    let end_tempo: i64 = if args.end > args.start {args.end as i64} else {tempo};
    let increase: i64 = args.increase as i64;
    let decrease: i64 = args.decrease as i64;
    let sweep: bool = args.wave;
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
        sweep,
    );
}

fn decode(composition: String) -> Result<Vec<(f64, usize, char)>, DecodeError> {
    let mut bar: Vec<(f64, usize, char)> = Vec::new();

    for p in composition
        .trim()
        .split(' ')
        .map(|x| format!("{:0>4}", x.trim()))
        .collect::<Vec<String>>() {
        if p.len() != 4 {
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
        let i: usize = match &p[2..3] {
            "k" => 0,
            "m" => 1,
            "h" => 2,
            "p" => 3,
            _   => return Err(DecodeError("illegal sound found, should be one of k, m, h, p"
                .to_string())),
        };
        let c: char = match &p[3..4] {
            "p" => '*',
            "s" => '\0',
            _   => return Err(DecodeError("illegal play indicator found, should be one of p, s"
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
    sweep: bool,
) {
    let start_tempo = tempo;
    let mut last_tempo = tempo;
    let start_bars = bars;

    let mut incrementor = [increase; 2];
    if decrease > 0 {
        incrementor[1] = decrease * -1;
    } else if sweep {
        incrementor[1] *= -1;
    }
    let mut alternator: usize = 0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut v_sink: Vec<Sink> = Vec::new();
    for _ in &bar {
        v_sink.push(Sink::try_new(&stream_handle).unwrap());
    }

    let sound: [Sound;3] = [
        Sound::get(kick()).unwrap(),
        Sound::get(kick_hi_hat()).unwrap(),
        Sound::get(hi_hat()).unwrap(),
    ];

    println!("Tempo: {}, Bars: {}", tempo, bars);

    let mut start = std::time::Instant::now();
    let mut note: f64 = 0f64;
    loop {

        let sixteenth: f64 = (60000f64 / tempo as f64) / 4f64;

        for _ in 0..bars {
            for (i, n) in bar.iter().enumerate() {
                let delay = Duration::from_millis((note * sixteenth) as u64);
                thread::sleep(delay.saturating_sub(std::time::Instant::now() - start));
                start = std::time::Instant::now();
                if n.1 < 3 {
                    print!("{}", n.2);
                    io::stdout().flush().unwrap();
                    v_sink[i].append(sound[n.1].decoder());
                }
                note = n.0;
            }
            println!();
        }

        tempo += incrementor[alternator];

        if sweep {
            if tempo > end_tempo {
                alternator = 1;
                tempo = end_tempo;
                tempo += incrementor[alternator];
            } else if tempo < start_tempo {
                alternator = 0;
                tempo = start_tempo;
                tempo += incrementor[alternator];
            }
        } else {
            alternator = alternator ^ 1;
            if tempo > end_tempo || tempo < start_tempo {
                break;
            }
        }

        bars = f64::round(tempo as f64 / (start_tempo * start_bars) as f64) as i64;

        if tempo != last_tempo {
            println!("Tempo: {}, Bars: {}", tempo, bars);
        }
        last_tempo = tempo;
    }

    thread::sleep(Duration::from_millis(500));
}
