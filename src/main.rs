mod sound;
mod configuration;
mod scheduling;
mod metronome;

use clap::Parser;
use crate::configuration::{build_config};
use crate::metronome::metronome;
use crate::scheduling::{schedule};

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
/// - Sound to use - k (kick drum), h (hi-hat), m (kick and hi-hat), s (stick), p (silent pause)
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
    /// Lower quarter note tempo (20-500)
    #[arg(short, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    lower: u16,

    /// Upper quarter note tempo (20-500)
    #[arg(short, default_value_t = 60, value_parser = clap::value_parser!(u16).range(20..501))]
    upper: u16,

    /// Increase step (alternates with a decrease if decrease is greater than zero)
    #[arg(short, default_value_t = 0)]
    increase: u8,

    /// Decrease step (alternates with an increase if increase is greater than zero)
    #[arg(short, default_value_t = 0)]
    decrease: u8,

    /// Number of bars to play in each tempo
    #[arg(short, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    n_bars: u8,

    /// Burst mode where upper tempo is burst tempo and lower tempo is normal tempo
    #[arg(short, default_value_t = 0)]
    burst: u8,

    /// Sweep from start to end and back in loop, ignored if in burst mode
    #[arg(short)]
    sweep: bool,

    /// Adaptive segment length increases length when tempo goes up
    #[arg(short)]
    adaptive: bool,

    /// Composition of bar
    #[arg(short, default_value = "4kp 4hp 4hp 4hp")]
    composition: String,

    /// Warn before tempo change with optional warn bar composition
    #[arg(short)]
    warn: Option<Option<String>>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let conf = build_config(args)?;
    let sched = schedule(&conf);

    metronome(sched, conf.max_ticks);

    Ok(())
}
