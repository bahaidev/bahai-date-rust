//! Baha'i Date Converter CLI
//!
//! A simple tool to convert between Gregorian and Badí' (Bahá'í) calendar dates.

use badi_date::{BadiDateLike, BadiMonth, Coordinates, FromDateTime, LocalBadiDate, LocalBadiDateLike, ToDateTime};
use chrono::TimeZone;
use chrono_tz::Tz;
use clap::{Parser, Subcommand};

/// Default location: Bahji
const DEFAULT_LATITUDE: f64 = 32.9434;
const DEFAULT_LONGITUDE: f64 = 35.0924;
const DEFAULT_TIMEZONE: &str = "Asia/Jerusalem";

#[derive(Parser)]
#[command(name = "bahai-date")]
#[command(version, about = "Convert between Gregorian and Baha'i dates", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Timezone (e.g., "America/New_York", "Asia/Jerusalem")
    #[arg(short, long, global = true)]
    timezone: Option<String>,

    /// Latitude for sunset calculation
    #[arg(short, long, global = true)]
    lat: Option<f64>,

    /// Longitude for sunset calculation
    #[arg(short = 'L', long, global = true)]
    lon: Option<f64>,

    /// Output using 'gum' and emojis for a pretty display
    #[arg(short, long, global = true)]
    fancy: bool,

    /// Show a Neofetch-style progress view of the year
    #[arg(short, long, global = true)]
    progress: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show today's Baha'i date (default if no command specified)
    Today,
    /// Convert Gregorian date to Baha'i date
    ToBadi {
        /// Year (e.g., 2024)
        #[arg(short = 'Y', long)]
        year: i32,
        /// Month (1-12)
        #[arg(short = 'M', long)]
        month: u32,
        /// Day (1-31)
        #[arg(short = 'D', long)]
        day: u32,
        /// Hour (0-23, optional)
        #[arg(short = 'H', long)]
        hour: Option<u32>,
        /// Minute (0-59, optional)
        #[arg(short = 'I', long)]
        minute: Option<u32>,
    },
    /// Convert Baha'i date to Gregorian date
    ToGreg {
        /// Baha'i year (e.g., 181)
        #[arg(short = 'Y', long)]
        year: u16,
        /// Baha'i month (0=Ayyam-i-Ha, 1-19)
        #[arg(short = 'M', long)]
        month: u8,
        /// Baha'i day (1-19)
        #[arg(short = 'D', long)]
        day: u8,
    },
}

fn main() {
    let cli = Cli::parse();

    let tz: Tz = cli
        .timezone
        .as_ref()
        .map(|s| s.replace(' ', "_"))
        .unwrap_or_else(|| DEFAULT_TIMEZONE.to_string())
        .parse()
        .unwrap_or_else(|_| DEFAULT_TIMEZONE.parse().unwrap());

    let lat = cli.lat.unwrap_or(DEFAULT_LATITUDE);
    let lon = cli.lon.unwrap_or(DEFAULT_LONGITUDE);
    let coords = Coordinates::new(lat, lon);

    if cli.progress {
        show_progress(tz, coords.ok());
        return;
    }

    match cli.command.unwrap_or(Commands::Today) {
        Commands::Today => {
            if cli.fancy {
                show_fancy_today(tz, coords.ok());
            } else {
                show_today(tz, coords.ok());
            }
        }
        Commands::ToBadi { year, month, day, hour, minute } => {
            greg_to_badi(year, month, day, hour.unwrap_or(12), minute.unwrap_or(0), tz, coords.ok(), cli.fancy);
        }
        Commands::ToGreg { year, month, day } => {
            badi_to_greg(year, month, day, tz, coords.ok(), cli.fancy);
        }
    }
}

fn show_fancy_today(tz: Tz, coords: Option<Coordinates>) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => {
            show_fancy_badi(&badi, now);
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_fancy_badi(badi: &LocalBadiDate, dt: chrono::DateTime<Tz>) {
    let month = badi.month();
    let month_name = month.transliteration();
    let day = badi.day();
    let year = badi.year();
    let greg_date = dt.format("%Y-%m-%d").to_string();
    let greg_time = dt.format("%H:%M %Z").to_string();
    let tz_name = badi.timezone().name();

    let emoji = match month_name.as_str() {
        "Bahá" => "✨",
        "Jalál" => "🌟",
        "Jamál" => "🌸",
        "Azamat" => "🏛️",
        "Núr" => "💡",
        "Rahmat" => "🤲",
        "Kalimát" => "📖",
        "Kamál" => "🎭",
        "Asmá" => "📛",
        "Izzat" => "💪",
        "Mashíyyat" => "🎯",
        "Ilm" => "📚",
        "Qudrat" => "⚡",
        "Qawl" => "💬",
        "Masá'il" => "❓",
        "Sharaf" => "🏅",
        "Sultán" => "👑",
        "Mulk" => "🌍",
        "Alá" => "🕊️",
        "Ayyám-i-Há" | "Ayyam-i-Ha" => "🎉",
        _ => "📅",
    };

    let header = format!("{} {}", emoji, month_name);
    let subheader = format!("Day {}  •  Year {}", day, year);
    let greg_info = format!("📅 {} {}", greg_date, greg_time);
    let tz_info = format!("🕐 {}", tz_name);

    let _ = std::process::Command::new("gum")
        .arg("style")
        .arg("--border")
        .arg("rounded")
        .arg("--border-foreground")
        .arg("99")
        .arg("--padding")
        .arg("1 2")
        .arg("--margin")
        .arg("0")
        .arg(format!("{}", gum_style(&header, "141", true)))
        .arg(format!("{}", gum_style(&subheader, "183", false)))
        .arg("")
        .arg(format!("{}", gum_style(&greg_info, "247", false)))
        .arg(format!("{}", gum_style(&tz_info, "247", false)))
        .status();
}

fn show_today(tz: Tz, coords: Option<Coordinates>) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => print_badi_date(&badi, now),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_progress(tz: Tz, coords: Option<Coordinates>) {
    let now = chrono::Utc::now().with_timezone(&tz);
    let badi = match LocalBadiDate::from_datetime(now, coords) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let day_start = badi.start();
    let day_end = badi.end();
    let day_progress = (now.timestamp() - day_start.timestamp()) as f64 / (day_end.timestamp() - day_start.timestamp()) as f64;

    let month_progress = (badi.day() as f64 - 1.0 + day_progress) / 19.0;

    // For year progress, we need the start of the year (Naw-Ruz)
    let year_start = LocalBadiDate::new(badi.year(), BadiMonth::Month(1), 1, tz, coords).unwrap().start();
    let next_year_start = LocalBadiDate::new(badi.year() + 1, BadiMonth::Month(1), 1, tz, coords).unwrap().start();
    let year_progress = (now.timestamp() - year_start.timestamp()) as f64 / (next_year_start.timestamp() - year_start.timestamp()) as f64;

    let year_u16 = badi.year() as u16;
    let year_in_vahid = ((year_u16 - 1) % 19) + 1;
    let vahid_progress = (year_in_vahid as f64 - 1.0 + year_progress) / 19.0;

    let vahid_in_kull_i_shay = (((year_u16 - 1) / 19) % 19) + 1;
    let kull_i_shay_progress = (vahid_in_kull_i_shay as f64 - 1.0 + vahid_progress) / 19.0;

    let vahid = ((year_u16 - 1) / 19) + 1;
    let kull_i_shay = ((year_u16 - 1) / 361) + 1;

    println!("\x1B[2J\x1B[H"); // Clear screen
    println!("{}", gum_style("Badi Year in Progress", "141", true));
    println!("{}", "─".repeat(40));

    print_progress_bar("Day", day_progress, &format!("Day {}", badi.day()));
    print_progress_bar("Month", month_progress, &badi.month().transliteration());
    print_progress_bar("Year", year_progress, &format!("Year {}", badi.year()));
    print_progress_bar("Vahid", vahid_progress, &format!("Vahid {}", vahid));
    print_progress_bar("Epoch", kull_i_shay_progress, &format!("Kull-i-Shay {}", kull_i_shay));

    println!("\n{}", gum_style("Press Enter to dismiss...", "242", false));
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

fn print_progress_bar(label: &str, progress: f64, value_text: &str) {
    let width = 50;
    let filled = (progress * width as f64).round() as usize;
    let empty = width - filled;
    
    let bar = format!(
        "\x1B[38;5;99m{}\x1B[0m{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );
    
    println!("{:<10} [{}] {:>5.1}% ({})", label, bar, progress * 100.0, value_text);
}

fn gum_style(text: &str, foreground: &str, bold: bool) -> String {
    let mut cmd = std::process::Command::new("gum");
    cmd.arg("style").arg("--foreground").arg(foreground);
    if bold {
        cmd.arg("--bold");
    }
    cmd.arg(text);

    let output = cmd.output().expect("failed to execute gum");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn greg_to_badi(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: Tz, coords: Option<Coordinates>, fancy: bool) {
    match tz.with_ymd_and_hms(year, month, day, hour, minute, 0).single() {
        Some(dt) => match LocalBadiDate::from_datetime(dt, coords) {
            Ok(badi) => {
                if fancy {
                    show_fancy_badi(&badi, dt);
                } else {
                    print_badi_date(&badi, dt);
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        },
        None => eprintln!("Error: Invalid date"),
    }
}

fn badi_to_greg(year: u16, month: u8, day: u8, tz: Tz, coords: Option<Coordinates>, fancy: bool) {
    let badi_month = if month == 0 {
        BadiMonth::AyyamIHa
    } else {
        BadiMonth::Month(month)
    };

    match LocalBadiDate::new(year as u8, badi_month, day as u16, tz, coords) {
        Ok(badi) => {
            let start = badi.start();
            if fancy {
                show_fancy_badi(&badi, start);
            } else {
                println!("Baha'i: {} {} {}", day, badi_month.transliteration(), year);
                println!("Gregorian: {} (starts at sunset)", start.format("%Y-%m-%d %H:%M %Z"));
            }
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn print_badi_date(badi: &LocalBadiDate, dt: chrono::DateTime<Tz>) {
    let month = badi.month();
    let month_name = month.transliteration();

    println!("{} {} {}", badi.day(), month_name, badi.year());
    println!("Gregorian: {} {}", dt.format("%Y-%m-%d"), dt.format("%H:%M %Z"));
    println!("Timezone: {}", badi.timezone().name());
}
