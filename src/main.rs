use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use devlapse::config::{default_output_dir, parse_args, ParseOutcome, USAGE};
use devlapse::scheduler::{run, SystemClock};
use devlapse::session::Session;
use devlapse::storage::Storage;
use devlapse::xcap_capturer::XcapCapturer;

fn main() -> ExitCode {
    let cfg = match parse_args(std::env::args().skip(1), default_output_dir()) {
        Ok(ParseOutcome::Run(c)) => c,
        Ok(ParseOutcome::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    // Fail fast on an unusable output dir instead of erroring every interval.
    if let Err(e) = std::fs::create_dir_all(&cfg.output_dir) {
        eprintln!("error: cannot create {}: {e}", cfg.output_dir.display());
        return ExitCode::FAILURE;
    }

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        if let Err(e) = ctrlc::set_handler(move || stop.store(true, Ordering::SeqCst)) {
            eprintln!("warning: Ctrl+C handler unavailable ({e}); kill the process to stop");
        }
    }

    println!("devlapse: every {:?} → {}", cfg.interval, cfg.output_dir.display());
    if let Some(h) = cfg.active_hours {
        println!("          active {}–{}", h.start.format("%H:%M"), h.end.format("%H:%M"));
    }
    println!("          Ctrl+C to stop");

    let mut session = Session::new(
        XcapCapturer::new(cfg.format),
        Storage::new(cfg.output_dir.clone()),
        cfg.format.extension(),
        cfg.active_hours,
    );

    let mut total = 0usize;
    let mut paused = false;
    run(&mut session, &SystemClock::new(), cfg.interval, &stop, cfg.max_captures, |r| {
        let now = chrono::Local::now().format("%H:%M:%S");
        if r.skipped_inactive {
            if !paused {
                println!("[{now}] outside active hours, waiting…");
                paused = true;
            }
            return;
        }
        paused = false;
        for p in &r.saved {
            println!("[{now}] saved {}", p.display());
        }
        for e in &r.errors {
            eprintln!("[{now}] {e}");
        }
        total += r.saved.len();
    });

    println!("Stopped. {total} screenshot(s) saved under {}", cfg.output_dir.display());
    ExitCode::SUCCESS
}
