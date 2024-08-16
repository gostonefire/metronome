use crate::configuration::Configuration;

pub struct Schedule {
    pub(crate) tempo: i64,
    pub(crate) bars: i64,
    pub(crate) bar: Vec<(f64, usize, char)>,
    pub(crate) lf: char,
    pub(crate) stop: bool,
}

pub fn schedule(conf: &Configuration) -> Vec<Schedule> {
    let mut sched: Vec<Schedule> = Vec::new();
    let mut bars = conf.bars;

    let mut tempo = conf.lower_tempo;
    let mut last_tempo = conf.lower_tempo;
    let start_tempo = conf.lower_tempo;
    let start_bars = bars;

    let mut incrementor = [conf.increase; 2];
    if conf.decrease > 0 {
        incrementor[1] = conf.decrease * -1;
    } else if conf.sweep {
        incrementor[1] *= -1;
    }
    let mut alternator: usize = 0;

    add_bar(&conf, tempo, bars, &mut sched);

    if conf.burst > 0 {
        if conf.warn && tempo != conf.upper_tempo {
            add_warn_bar(&conf, conf.upper_tempo, &mut sched);
        }

        add_bar(&conf, conf.upper_tempo, conf.burst, &mut sched);

    } else {
        loop {
            tempo += incrementor[alternator];

            if conf.sweep {
                if tempo > conf.upper_tempo {
                    alternator = 1;
                    tempo = conf.upper_tempo;
                    tempo += incrementor[alternator];
                } else if tempo <= start_tempo {
                    break;
                }
            } else {
                alternator = alternator ^ 1;
                if tempo > conf.upper_tempo || tempo < start_tempo {
                    add_stop_bar(&conf, &mut sched);
                    break;
                }
            }

            if conf.adaptive {
                bars = f64::round(tempo as f64 / (start_tempo * start_bars) as f64) as i64;
            }

            if conf.warn && tempo != last_tempo {
                add_warn_bar(&conf, tempo, &mut sched);
            }

            add_bar(&conf, tempo, bars, &mut sched);

            last_tempo = tempo;
        }
    }

    if conf.warn && !sched[sched.len() - 1].stop && sched[0].tempo != sched[sched.len() - 1].tempo {
        add_warn_bar(&conf, sched[0].tempo, &mut sched);
    }
    return sched;
}

fn add_bar(conf: &Configuration, tempo: i64, bars: i64, sched: &mut Vec<Schedule>) {
    sched.push(Schedule {
        tempo,
        bars,
        bar: conf.bar.clone(),
        lf: conf.bar_lf,
        stop: false,
    });
}

fn add_warn_bar(conf: &Configuration, tempo: i64, sched: &mut Vec<Schedule>) {
    sched.push(Schedule {
        tempo,
        bars: 1,
        bar: conf.warn_bar.clone(),
        lf: conf.warn_bar_lf,
        stop: false,
    });
}

fn add_stop_bar(conf: &Configuration, sched: &mut Vec<Schedule>) {
    sched.push(Schedule {
        tempo: 0,
        bars: 0,
        bar: conf.bar.clone(),
        lf: conf.bar_lf,
        stop: true,
    });
}