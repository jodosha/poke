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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    #[cfg(target_os = "macos")]
    {
        // Pin to our own bundle id so macOS persists permission and doesn't
        // pop the "Choose Application" picker.
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

    n.show()?;
    Ok(())
}
