use clap::Parser;
use notify_rust::{Notification, Timeout};

#[derive(Parser, Debug)]
#[command(name = "poke", about = "Deliver macOS notifications")]
struct Args {
    #[arg(long)]
    title: String,

    #[arg(long)]
    message: String,

    /// Seconds before the notification disappears. 0 = sticky.
    #[arg(long, default_value_t = 5)]
    timeout: u32,

    /// low | normal | high
    #[arg(long, default_value = "normal")]
    severity: String,

    /// Path or URL opened when the notification is clicked (xdg-only).
    #[arg(long)]
    target: Option<String>,

    /// Schedule delivery for later. Compound durations: 30m, 1h, 90s, 1h30m, 2d.
    /// macOS only.
    #[arg(long, value_name = "DURATION")]
    r#in: Option<String>,
}

/// Parse `1h30m`, `90s`, `2d`, etc. Returns total seconds.
fn parse_duration(s: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err("empty duration".into());
    }
    let mut total: u64 = 0;
    let mut num: u64 = 0;
    let mut saw_digit = false;
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            num = num
                .checked_mul(10)
                .and_then(|n| n.checked_add(d as u64))
                .ok_or_else(|| format!("duration overflow in {s:?}"))?;
            saw_digit = true;
            continue;
        }
        if !saw_digit {
            return Err(format!("expected digit before {c:?} in {s:?}"));
        }
        let mult: u64 = match c {
            's' | 'S' => 1,
            'm' | 'M' => 60,
            'h' | 'H' => 3_600,
            'd' | 'D' => 86_400,
            _ => return Err(format!("unknown unit {c:?} in {s:?}; use s/m/h/d")),
        };
        total = total
            .checked_add(num.checked_mul(mult).ok_or("duration overflow")?)
            .ok_or("duration overflow")?;
        num = 0;
        saw_digit = false;
    }
    if saw_digit {
        return Err(format!("missing unit at end of {s:?}; use s/m/h/d"));
    }
    if total == 0 {
        return Err(format!("duration {s:?} resolves to zero"));
    }
    Ok(total)
}

/// Marker env var set on the detached child so we don't re-spawn forever.
const DETACHED_ENV: &str = "POKE_DETACHED";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // notify-rust's macOS schedule_raw blocks until delivery. When --in is set
    // and we're the parent invocation, re-spawn ourselves detached so the
    // caller's shell isn't held for the full delay.
    #[cfg(target_os = "macos")]
    if args.r#in.is_some() && std::env::var_os(DETACHED_ENV).is_none() {
        let exe = std::env::current_exe()?;
        let argv: Vec<String> = std::env::args().skip(1).collect();
        std::process::Command::new(exe)
            .args(&argv)
            .env(DETACHED_ENV, "1")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let _ = notify_rust::set_application("com.lucaguidi.poke");
    }

    let mut n = Notification::new();
    n.summary(&args.title).body(&args.message);

    n.timeout(if args.timeout == 0 {
        Timeout::Never
    } else {
        Timeout::Milliseconds(args.timeout * 1000)
    });

    #[cfg(not(target_os = "macos"))]
    {
        use notify_rust::{Hint, Urgency};
        let urgency = match args.severity.to_ascii_lowercase().as_str() {
            "low" => Urgency::Low,
            "high" => Urgency::Critical,
            _ => Urgency::Normal,
        };
        n.hint(Hint::Urgency(urgency));

        if args.r#in.is_some() {
            return Err("--in is a macOS-only feature".into());
        }

        if let Some(target) = args.target.as_deref() {
            n.action("default", "Open");
            n.hint(Hint::Resident(true));
            n.show()?.wait_for_action(|action| {
                if action == "default" {
                    let _ = std::process::Command::new("xdg-open").arg(target).status();
                }
            });
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    if args.target.is_some() {
        eprintln!("poke: --target is a freedesktop-only feature; ignoring on macOS");
    }

    #[cfg(target_os = "macos")]
    if let Some(spec) = args.r#in.as_deref() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = parse_duration(spec).map_err(|e| format!("--in: {e}"))?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        n.schedule_raw(now + secs as f64)?;
        return Ok(());
    }

    n.show()?;
    Ok(())
}
