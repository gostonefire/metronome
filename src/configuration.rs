use crate::Args;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct DecodeError(String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Composition decode error: {}", self.0)
    }
}

pub struct Configuration {
    pub(crate) lower_tempo: i64,
    pub(crate) upper_tempo: i64,
    pub(crate) increase: i64,
    pub(crate) decrease: i64,
    pub(crate) bar: Vec<(f64, usize, char)>,
    pub(crate) bar_lf: char,
    pub(crate) warn_bar: Vec<(f64, usize, char)>,
    pub(crate) warn_bar_lf: char,
    pub(crate) bars: i64,
    pub(crate) burst: i64,
    pub(crate) sweep: bool,
    pub(crate) adaptive: bool,
    pub(crate) warn: bool,
    pub(crate) max_ticks: usize,
}

pub fn build_config(args: Args) -> Result<Configuration, String> {
    let lower_tempo: i64 = args.lower as i64;
    let upper_tempo: i64 = if args.upper > args.lower {
        args.upper as i64
    } else {
        lower_tempo
    };
    let increase: i64 = args.increase as i64;
    let decrease: i64 = args.decrease as i64;
    let sweep: bool = args.sweep;
    let composition: String = args.composition.unwrap_or_else(|| "4kp 4hp 4hp 4hp".to_string());
    let bars: i64 = args.n_bars as i64;
    let adaptive: bool = args.adaptive;
    let burst: i64 = args.burst as i64;

    let bar = match decode(composition) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Bar composition: {}", e));
        }
    };

    let mut bar_lf = '\0';
    for b in &bar {
        if b.2 != '\0' {
            bar_lf = '\n';
            break;
        }
    }

    let mut warn = false;
    let mut warn_comp = "4ss 4ss 4ss 4ss".to_string();

    match args.warn {
        Some(w) => {
            match w {
                Some(c) => {
                    warn_comp = c;
                }
                None => (),
            }
            warn = true;
        }
        None => (),
    }
    let warn_bar = match decode(warn_comp) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Warn bar composition: {}", e));
        }
    };

    let mut warn_bar_lf = '\0';
    for b in &warn_bar {
        if b.2 != '\0' {
            warn_bar_lf = '\n';
            break;
        }
    }

    let max_ticks = bar.len().max(warn_bar.len());

    Ok(Configuration {
        lower_tempo,
        upper_tempo,
        increase,
        decrease,
        bar,
        bar_lf,
        warn_bar,
        warn_bar_lf,
        bars,
        burst,
        sweep,
        adaptive,
        warn,
        max_ticks,
    })
}

fn decode(composition: String) -> Result<Vec<(f64, usize, char)>, DecodeError> {
    let mut bar: Vec<(f64, usize, char)> = Vec::new();

    for p in composition
        .split_whitespace()
        .map(|x| format!("{:0>4}", x))
        .collect::<Vec<String>>()
    {
        if p.len() != 4 {
            return Err(DecodeError("malformed beat definition".to_string()));
        }

        let n = u8::from_str(&p[0..2]).map_err(|_| {
            DecodeError("illegal note found, should be one of 1, 2, 4, 8, 16".to_string())
        })?;
        let t: f64 = match n {
            1 => 16f64,
            2 => 8f64,
            4 => 4f64,
            8 => 2f64,
            16 => 1f64,
            _ => {
                return Err(DecodeError(
                    "illegal note found, should be one of 1, 2, 4, 8, 16".to_string(),
                ))
            }
        };
        let i: usize = match &p[2..3] {
            "k" => 0,
            "m" => 1,
            "h" => 2,
            "s" => 3,
            "p" => 4,
            _ => {
                return Err(DecodeError(
                    "illegal sound found, should be one of k, m, h, p".to_string(),
                ))
            }
        };
        let c: char = match &p[3..4] {
            "p" => '*',
            "s" => '\0',
            _ => {
                return Err(DecodeError(
                    "illegal play indicator found, should be one of p, s".to_string(),
                ))
            }
        };

        bar.push((t, i, c));
    }

    if bar.len() == 0 {
        Err(DecodeError("empty composition".to_string()))
    } else {
        Ok(bar)
    }
}
