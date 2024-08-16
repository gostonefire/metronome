use crate::scheduling::Schedule;
use crate::sound::{hi_hat, kick, kick_hi_hat, Sound, sticks};
use rodio::{OutputStream, Sink};
use std::io::Write;
use std::time::Duration;
use std::{io, thread};

pub fn metronome(sched: Vec<Schedule>, max_ticks: usize) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut v_sink: Vec<Sink> = Vec::new();
    for _ in 0..max_ticks {
        v_sink.push(Sink::try_new(&stream_handle).unwrap());
    }

    let sound: [Sound; 4] = [
        Sound::get(kick()).unwrap(),
        Sound::get(kick_hi_hat()).unwrap(),
        Sound::get(hi_hat()).unwrap(),
        Sound::get(sticks()).unwrap(),
    ];

    let mut start = std::time::Instant::now();
    let mut note: f64 = 0f64;
    let mut last_tempo = 0i64;

    'outer: loop {
        for b in 0..sched.len() {
            let s = &sched[b];
            if s.stop {
                break 'outer;
            }

            if s.tempo != last_tempo {
                println!("Tempo: {}", s.tempo);
                last_tempo = s.tempo;
            }

            let sixteenth: f64 = (60000f64 / s.tempo as f64) / 4f64;

            for _ in 0..s.bars {
                for (i, n) in s.bar.iter().enumerate() {
                    let delay = Duration::from_millis((note * sixteenth) as u64);
                    thread::sleep(delay.saturating_sub(std::time::Instant::now() - start));
                    start = std::time::Instant::now();
                    if n.1 < 4 {
                        print!("{}", n.2);
                        io::stdout().flush().unwrap();
                        v_sink[i].append(sound[n.1].decoder());
                    }
                    note = n.0;
                }
                print!("{}", s.lf);
            }
        }
    }

    thread::sleep(Duration::from_millis(500));
}
