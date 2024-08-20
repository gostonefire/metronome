use crate::configuration::Configuration;

pub struct Schedule {
    pub(crate) tempo: i64,
    pub(crate) bars: i64,
    pub(crate) bar: Vec<(f64, usize, char)>,
    pub(crate) lf: char,
    pub(crate) stop: bool,
}

pub fn schedule(conf: &Configuration) -> Result<Vec<Schedule>, String> {
    let mut sched: Vec<Schedule> = Vec::new();

    if conf.increase > conf.upper_tempo - conf.lower_tempo {
        return Err("Increase argument can't be higher than difference between low and high".to_string());
    }

    if conf.decrease > conf.upper_tempo - conf.lower_tempo {
        return Err("Decrease argument can't be higher than difference between low and high".to_string());
    }

    if conf.sweep {
        if conf.increase == 0 || conf.decrease == 0 {
            return Err("Both increase and decrease argument must be non zero when in sweep mode".to_string());
        }

        schedule_sweep(conf, &mut sched);
    } else if conf.burst > 0 {

        schedule_burst(conf, &mut sched);
    } else if conf.increase == conf.decrease {

        schedule_loop(conf, &mut sched);
    } else {
        if conf.increase < conf.decrease {
            return Err("Increase must be larger or equal to decrease (unless in sweep mode)".to_string());
        }

        schedule_increase(conf, &mut sched);
    }

    Ok(sched)
}

fn schedule_increase(conf: &Configuration, sched: &mut Vec<Schedule>) {
    let mut bars = conf.bars;
    let mut tempo = conf.lower_tempo;
    let mut last_tempo = conf.lower_tempo;

    let mut incrementor = [conf.increase; 2];
    if conf.decrease > 0 {
        incrementor[1] = conf.decrease * -1;
    } else if conf.sweep {
        incrementor[1] *= -1;
    }
    let mut alternator: usize = 0;

    add_bar(&conf, tempo, bars, sched);

    loop {
        tempo += incrementor[alternator];

        alternator = alternator ^ 1;
        if tempo > conf.upper_tempo || tempo < conf.lower_tempo {
            add_stop_bar(&conf, sched);
            break;
        }

        if conf.adaptive {
            bars = f64::round(tempo as f64 / (conf.lower_tempo * conf.bars) as f64) as i64;
        }

        if conf.warn && tempo != last_tempo {
            add_warn_bar(&conf, tempo, sched);
        }

        add_bar(&conf, tempo, bars, sched);

        last_tempo = tempo;
    }
}

fn schedule_sweep(conf: &Configuration, sched: &mut Vec<Schedule>) {
    let mut bars = conf.bars;
    let mut tempo = conf.lower_tempo;
    let mut last_tempo = conf.lower_tempo;

    let mut incrementor = conf.increase;

    add_bar(&conf, tempo, bars, sched);

    loop {
        tempo += incrementor;

        if tempo > conf.upper_tempo {
            incrementor = conf.decrease * -1;
            tempo = conf.upper_tempo;
            tempo += incrementor;
        } else if tempo <= conf.lower_tempo {
            break;
        }

        if conf.adaptive {
            bars = f64::round(tempo as f64 / (conf.lower_tempo * conf.bars) as f64) as i64;
        }

        if conf.warn && tempo != last_tempo {
            add_warn_bar(&conf, tempo, sched);
        }

        add_bar(&conf, tempo, bars, sched);

        last_tempo = tempo;
    }

    if conf.warn {
        add_warn_bar(&conf, sched[0].tempo, sched);
    }
}

fn schedule_burst(conf: &Configuration, sched: &mut Vec<Schedule>) {
    add_bar(&conf, conf.lower_tempo, conf.bars, sched);

    if conf.warn && conf.lower_tempo != conf.upper_tempo {
        add_warn_bar(&conf, conf.upper_tempo, sched);
    }

    add_bar(&conf, conf.upper_tempo, conf.burst, sched);

    if conf.warn && conf.lower_tempo != conf.upper_tempo  {
        add_warn_bar(&conf, sched[0].tempo, sched);
    }
}

fn schedule_loop(conf: &Configuration, sched: &mut Vec<Schedule>) {
    add_bar(&conf, conf.lower_tempo, conf.bars, sched);

    if conf.warn && conf.increase > 0 {
        add_warn_bar(&conf, conf.lower_tempo + conf.increase, sched);
    }

    add_bar(&conf, conf.lower_tempo + conf.increase, conf.bars, sched);

    if conf.warn && conf.increase > 0  {
        add_warn_bar(&conf, sched[0].tempo, sched);
    }
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