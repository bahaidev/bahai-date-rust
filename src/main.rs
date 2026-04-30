//! Baha'i Date Converter CLI
//!
//! A simple tool to convert between Gregorian and Badi' (Baha'i) calendar dates.

use badi_date::{
    BadiDate, BadiDateLike, BadiMonth, BahaiHolyDay, Coordinates, FromDateTime,
    HolyDayProviding, LocalBadiDate, LocalBadiDateLike, ToDateTime,
};
use chrono::TimeZone;
use chrono_tz::Tz;
use clap::{Parser, Subcommand};

/// Default location: Bahji
const DEFAULT_LATITUDE: f64 = 32.9434;
const DEFAULT_LONGITUDE: f64 = 35.0924;
const FALLBACK_TIMEZONE: &str = "Asia/Jerusalem";

fn system_timezone() -> String {
    iana_time_zone::get_timezone().unwrap_or_else(|_| FALLBACK_TIMEZONE.to_string())
}

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
    #[arg(short, long, global = true, num_args = 0..=1, default_missing_value = "all")]
    progress: Option<String>,

    /// Countdown to the next time bucket (e.g., "day", "month", "holy-day", "year")
    #[arg(short = 'c', long, visible_alias = "cd", global = true, num_args = 0..=1, default_missing_value = "all")]
    countdown: Option<String>,
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
    /// Show the next upcoming holy day with date and celebration time
    NextHolyDay,
    /// List all holy days for the current Baha'i year
    HolyDays,
}

fn main() {
    let cli = Cli::parse();

    let tz: Tz = cli
        .timezone
        .as_ref()
        .map(|s| s.replace(' ', "_"))
        .unwrap_or_else(system_timezone)
        .parse()
        .unwrap_or_else(|_| FALLBACK_TIMEZONE.parse().unwrap());

    let lat = cli.lat.unwrap_or(DEFAULT_LATITUDE);
    let lon = cli.lon.unwrap_or(DEFAULT_LONGITUDE);
    let coords = Coordinates::new(lat, lon);

    if let Some(entry) = cli.progress {
        show_progress(tz, coords.ok(), &entry);
        return;
    }

    if let Some(bucket) = cli.countdown {
        show_countdown(tz, coords.ok(), &bucket);
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
        Commands::NextHolyDay => {
            show_next_holy_day(tz, coords.ok(), cli.fancy);
        }
        Commands::HolyDays => {
            show_holy_days(tz, coords.ok(), cli.fancy);
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
    let greg_info = format!("{} {}", greg_date, greg_time);
    let tz_info = format!("🕐 {}", tz_name);

    let mut args: Vec<String> = vec![
        gum_style(&header, "141", true),
        gum_style(&subheader, "183", false),
        String::new(),
        gum_style(&greg_info, "247", false),
        gum_style(&tz_info, "247", false),
    ];

    // Build annotation lines for holy day / Ayyam-i-Ha / feast
    let mut annotations = Vec::new();

    if let Ok(badi_date) = BadiDate::new(year, month, day) {
        if let Some(hd) = badi_date.holy_day() {
            let hd_emoji = holy_day_emoji(&hd);
            let work = if hd.work_suspended() {
                "work suspended"
            } else {
                "observance"
            };
            annotations.push(format!("{} {} ({})", hd_emoji, hd.english(), work));
        }
    }

    if month == BadiMonth::AyyamIHa {
        let total = BadiMonth::AyyamIHa.number_of_days(year);
        annotations.push(format!("🎉 Ayyam-i-Ha Day {} of {}", day, total));
    }

    if badi.is_feast() {
        annotations.push(format!("🍽️  Nineteen Day Feast of {}", month_name));
    }

    if !annotations.is_empty() {
        args.push(String::new());
        for ann in &annotations {
            args.push(gum_style(ann, "214", false));
        }
    }

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
        .args(&args)
        .status();
}

fn show_today(tz: Tz, coords: Option<Coordinates>) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => print_badi_date(&badi, now),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_progress(tz: Tz, coords: Option<Coordinates>, entry: &str) {
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
    let day_progress = (now.timestamp() - day_start.timestamp()) as f64
        / (day_end.timestamp() - day_start.timestamp()) as f64;

    let month_progress = (badi.day() as f64 - 1.0 + day_progress) / 19.0;

    // For year progress, we need the start of the year (Naw-Ruz)
    let year_start =
        LocalBadiDate::new(badi.year(), BadiMonth::Month(1), 1, tz, coords)
            .unwrap()
            .start();
    let next_year_start =
        LocalBadiDate::new(badi.year() + 1, BadiMonth::Month(1), 1, tz, coords)
            .unwrap()
            .start();
    let year_progress = (now.timestamp() - year_start.timestamp()) as f64
        / (next_year_start.timestamp() - year_start.timestamp()) as f64;

    let year_u16 = badi.year() as u16;
    let year_in_vahid = ((year_u16 - 1) % 19) + 1;
    let vahid_progress = (year_in_vahid as f64 - 1.0 + year_progress) / 19.0;

    let vahid_in_kull_i_shay = (((year_u16 - 1) / 19) % 19) + 1;
    let kull_i_shay_progress = (vahid_in_kull_i_shay as f64 - 1.0 + vahid_progress) / 19.0;

    let vahid = ((year_u16 - 1) / 19) + 1;
    let kull_i_shay = ((year_u16 - 1) / 361) + 1;

    if entry == "all" {
        println!("\x1B[2J\x1B[H"); // Clear screen
        println!("{}", gum_style("Badi Year in Progress", "141", true));
        println!("{}", "─".repeat(40));

        print_fancy_progress_bar("Day", day_progress, &format!("Day {}", badi.day()));
        print_fancy_progress_bar("Month", month_progress, &badi.month().transliteration());
        print_fancy_progress_bar("Year", year_progress, &format!("Year {}", badi.year()));
        print_fancy_progress_bar("Vahid", vahid_progress, &format!("Vahid {}", vahid));
        print_fancy_progress_bar(
            "Epoch",
            kull_i_shay_progress,
            &format!("Kull-i-Shay {}", kull_i_shay),
        );

        // Show next holy day info
        if let Ok(badi_date) = BadiDate::new(badi.year(), badi.month(), badi.day()) {
            if let Ok(next_hd) = badi_date.next_holy_day() {
                if let Ok(next_local) =
                    LocalBadiDate::new(next_hd.year(), next_hd.month(), next_hd.day(), tz, coords)
                {
                    let diff = (next_local.start().date_naive() - badi.start().date_naive())
                        .num_days();
                    if let Some(hd) = next_hd.holy_day() {
                        let hd_emoji = holy_day_emoji(&hd);
                        println!(
                            "\n{}",
                            gum_style(
                                &format!(
                                    "{} Next Holy Day: {} in {} days",
                                    hd_emoji,
                                    hd.english(),
                                    diff
                                ),
                                "214",
                                false,
                            )
                        );
                    }
                }
            }
        }

        println!("\n{}", gum_style("Press Enter to dismiss...", "242", false));
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    } else {
        match entry.to_lowercase().as_str() {
            "day" => {
                print_plain_progress_bar("Day", day_progress, &format!("Day {}", badi.day()))
            }
            "month" => {
                print_plain_progress_bar("Month", month_progress, &badi.month().transliteration())
            }
            "year" => {
                print_plain_progress_bar("Year", year_progress, &format!("Year {}", badi.year()))
            }
            "vahid" => {
                print_plain_progress_bar("Vahid", vahid_progress, &format!("Vahid {}", vahid))
            }
            "epoch" | "kull-i-shay" | "kullishay" => print_plain_progress_bar(
                "Epoch",
                kull_i_shay_progress,
                &format!("Kull-i-Shay {}", kull_i_shay),
            ),
            _ => eprintln!("Unknown progress entry: {}", entry),
        }
    }
}

fn show_countdown(tz: Tz, coords: Option<Coordinates>, bucket: &str) {
    let now = chrono::Utc::now().with_timezone(&tz);
    let badi = match LocalBadiDate::from_datetime(now, coords) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    if bucket == "all" {
        show_countdown_all(&badi, now, tz, coords);
        return;
    }

    match bucket.to_lowercase().as_str() {
        "day" => {
            let diff_secs = (badi.end().timestamp() - now.timestamp()).max(0);
            let hours = diff_secs as f64 / 3600.0;
            if hours < 1.0 {
                let mins = (hours * 60.0).round() as u32;
                println!("{} minutes", mins);
            } else {
                println!("{:.1} hours", hours);
            }
        }
        "month" | "feast" => {
            let diff = countdown_days_to_next_feast(&badi, tz, coords);
            println!("{} days", diff);
        }
        "holy-day" | "holyday" => {
            match countdown_days_to_next_holy_day(&badi, tz, coords) {
                Some(d) => println!("{} days", d),
                None => eprintln!("Error: Could not determine next holy day"),
            }
        }
        "year" | "nawruz" | "naw-ruz" => {
            let next_year_start =
                LocalBadiDate::new(badi.year() + 1, BadiMonth::Month(1), 1, tz, coords).unwrap();
            let diff =
                (next_year_start.start().date_naive() - badi.start().date_naive()).num_days();
            println!("{} days", diff);
        }
        "vahid" => {
            let next_vahid_year = ((((badi.year() as u16 - 1) / 19) + 1) * 19) + 1;
            if next_vahid_year > 255 {
                eprintln!(
                    "Error: Baha'i year {} exceeds the supported limit of 255.",
                    next_vahid_year
                );
                return;
            }
            let next_vahid_start = LocalBadiDate::new(
                next_vahid_year as u8,
                BadiMonth::Month(1),
                1,
                tz,
                coords,
            )
            .unwrap();
            let diff =
                (next_vahid_start.start().date_naive() - badi.start().date_naive()).num_days();
            println!("{} days", diff);
        }
        _ => eprintln!("Unknown countdown bucket: {}", bucket),
    }
}

fn show_countdown_all(
    badi: &LocalBadiDate,
    now: chrono::DateTime<Tz>,
    tz: Tz,
    coords: Option<Coordinates>,
) {
    println!();

    // Hours/minutes until next Badi day
    let diff_secs = (badi.end().timestamp() - now.timestamp()).max(0);
    let hours = diff_secs as f64 / 3600.0;
    if hours < 1.0 {
        let mins = (hours * 60.0).round() as u32;
        println!("\x1B[38;5;183m{}\x1B[0m minutes until next day", mins);
    } else {
        println!("\x1B[38;5;183m{:.1}\x1B[0m hours until next day", hours);
    }

    // Days until next feast
    let feast_diff = countdown_days_to_next_feast(badi, tz, coords);
    println!("\x1B[38;5;183m{}\x1B[0m days until next feast", feast_diff);

    // Days until next holy day (with name)
    if let Some(hd_diff) = countdown_days_to_next_holy_day(badi, tz, coords) {
        if let Ok(badi_date) = BadiDate::new(badi.year(), badi.month(), badi.day()) {
            if let Ok(next_hd) = badi_date.next_holy_day() {
                if let Some(hd) = next_hd.holy_day() {
                    println!(
                        "\x1B[38;5;214m{}\x1B[0m days until {} {}",
                        hd_diff,
                        holy_day_emoji(&hd),
                        hd.english()
                    );
                }
            }
        }
    }

    // Days until next Ayyam-i-Ha
    if badi.month() == BadiMonth::AyyamIHa {
        let total = BadiMonth::AyyamIHa.number_of_days(badi.year());
        let remaining = total as i64 - badi.day() as i64;
        println!("\x1B[38;5;183m{}\x1B[0m days remaining in Ayyam-i-Ha", remaining);
    } else {
        let ayh_year = if badi.month() == BadiMonth::Month(19) {
            badi.year() + 1
        } else {
            badi.year()
        };
        if let Ok(ayh_start) =
            LocalBadiDate::new(ayh_year, BadiMonth::AyyamIHa, 1, tz, coords)
        {
            let diff = (ayh_start.start().date_naive() - badi.start().date_naive()).num_days();
            println!("\x1B[38;5;183m{}\x1B[0m days until Ayyam-i-Ha", diff);
        }
    }

    // Days until next year (Naw-Ruz)
    let next_year_start =
        LocalBadiDate::new(badi.year() + 1, BadiMonth::Month(1), 1, tz, coords).unwrap();
    let year_diff = (next_year_start.start().date_naive() - badi.start().date_naive()).num_days();
    println!("\x1B[38;5;183m{}\x1B[0m days until Naw-Ruz", year_diff);

    // Years until next Vahid
    let year_u16 = badi.year() as u16;
    let vahid = ((year_u16 - 1) / 19) + 1;
    let years_in_vahid = ((year_u16 - 1) % 19) + 1;
    let years_until_vahid = 19 - years_in_vahid;
    if years_until_vahid == 0 {
        println!("Vahid \x1B[38;5;141m{}\x1B[0m begins this year", vahid + 1);
    } else {
        println!("\x1B[38;5;141m{}\x1B[0m years until Vahid \x1B[38;5;141m{}\x1B[0m", years_until_vahid, vahid + 1);
    }

    // Vahids until next Kull-i-Shay
    let kull_i_shay = ((year_u16 - 1) / 361) + 1;
    let vahid_in_kull = (((year_u16 - 1) / 19) % 19) + 1;
    let vahids_until_kull = 19 - vahid_in_kull;
    if vahids_until_kull == 0 {
        println!("Kull-i-Shay \x1B[38;5;141m{}\x1B[0m begins this Vahid", kull_i_shay + 1);
    } else {
        println!("\x1B[38;5;141m{}\x1B[0m vahids until Kull-i-Shay \x1B[38;5;141m{}\x1B[0m", vahids_until_kull, kull_i_shay + 1);
    }

    println!();
}

fn countdown_days_to_next_feast(
    badi: &LocalBadiDate,
    tz: Tz,
    coords: Option<Coordinates>,
) -> i64 {
    let (next_month, next_year) = match badi.month() {
        BadiMonth::Month(19) => (BadiMonth::Month(1), badi.year() + 1),
        BadiMonth::Month(m) => (BadiMonth::Month(m + 1), badi.year()),
        BadiMonth::AyyamIHa => (BadiMonth::Month(19), badi.year()),
    };
    let next_feast = LocalBadiDate::new(next_year, next_month, 1, tz, coords).unwrap();
    (next_feast.start().date_naive() - badi.start().date_naive()).num_days()
}

fn countdown_days_to_next_holy_day(
    badi: &LocalBadiDate,
    tz: Tz,
    coords: Option<Coordinates>,
) -> Option<i64> {
    let badi_date = BadiDate::new(badi.year(), badi.month(), badi.day()).ok()?;
    let next_hd = badi_date.next_holy_day().ok()?;
    let next_local =
        LocalBadiDate::new(next_hd.year(), next_hd.month(), next_hd.day(), tz, coords).ok()?;
    Some((next_local.start().date_naive() - badi.start().date_naive()).num_days())
}

fn print_fancy_progress_bar(label: &str, progress: f64, value_text: &str) {
    let width = 50;
    let filled = (progress * width as f64).round() as usize;
    let empty = width - filled;

    let bar = format!(
        "\x1B[38;5;99m{}\x1B[0m{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );

    println!(
        "{:<10} [{}] {:>5.1}% ({})",
        label, bar, progress * 100.0, value_text
    );
}

fn print_plain_progress_bar(label: &str, progress: f64, value_text: &str) {
    let width = 50;
    let filled = (progress * width as f64).round() as usize;
    let empty = width - filled;

    let bar = format!("{}{}", "#".repeat(filled), "-".repeat(empty));

    println!(
        "{:<10} [{}] {:>5.1}% ({})",
        label, bar, progress * 100.0, value_text
    );
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

fn greg_to_badi(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    tz: Tz,
    coords: Option<Coordinates>,
    fancy: bool,
) {
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

fn badi_to_greg(
    year: u16,
    month: u8,
    day: u8,
    tz: Tz,
    coords: Option<Coordinates>,
    fancy: bool,
) {
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
                println!(
                    "Baha'i: {} {} {}",
                    day,
                    badi_month.transliteration(),
                    year
                );
                println!(
                    "Gregorian: {} (starts at sunset)",
                    start.format("%Y-%m-%d %H:%M %Z")
                );
            }
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn print_badi_date(badi: &LocalBadiDate, dt: chrono::DateTime<Tz>) {
    let month = badi.month();
    let month_name = month.transliteration();

    println!("{} {} {}", badi.day(), month_name, badi.year());
    println!(
        "Gregorian: {} {}",
        dt.format("%Y-%m-%d"),
        dt.format("%H:%M %Z")
    );
    println!("Timezone: {}", badi.timezone().name());

    // Holy day annotation
    if let Ok(badi_date) = BadiDate::new(badi.year(), month, badi.day()) {
        if let Some(hd) = badi_date.holy_day() {
            let hd_emoji = holy_day_emoji(&hd);
            let work = if hd.work_suspended() {
                "work suspended"
            } else {
                "observance"
            };
            println!("{} {} ({})", hd_emoji, hd.english(), work);
        }
    }

    // Ayyam-i-Ha annotation
    if month == BadiMonth::AyyamIHa {
        let total = BadiMonth::AyyamIHa.number_of_days(badi.year());
        println!("Ayyam-i-Ha Day {} of {}", badi.day(), total);
    }

    // Feast day annotation
    if badi.is_feast() {
        println!("Nineteen Day Feast of {}", month_name);
    }
}

fn show_next_holy_day(tz: Tz, coords: Option<Coordinates>, fancy: bool) {
    let now = chrono::Utc::now().with_timezone(&tz);
    let badi = match LocalBadiDate::from_datetime(now, coords) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let badi_date = match BadiDate::new(badi.year(), badi.month(), badi.day()) {
        Ok(bd) => bd,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let next_hd = match badi_date.next_holy_day() {
        Ok(hd) => hd,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let hd = next_hd.holy_day().unwrap();
    let next_local =
        LocalBadiDate::new(next_hd.year(), next_hd.month(), next_hd.day(), tz, coords).unwrap();
    let greg_start = next_local.start();

    let diff = (next_local.start().date_naive() - badi.start().date_naive()).num_days();
    let hd_emoji = holy_day_emoji(&hd);
    let celeb_time = celebration_time(&hd);

    if fancy {
        let header = format!("{} {}", hd_emoji, hd.english());
        let badi_info = format!(
            "{} {} {}",
            next_hd.day(),
            next_hd.month().transliteration(),
            next_hd.year()
        );
        let greg_info = format!(
            "{} {}",
            greg_start.format("%Y-%m-%d"),
            greg_start.format("%H:%M %Z")
        );
        let countdown = format!("{} days from now", diff);
        let celeb = format!("Celebrated at: {}", celeb_time);

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
            .arg(gum_style(&header, "214", true))
            .arg(gum_style(&badi_info, "183", false))
            .arg(String::new())
            .arg(gum_style(&greg_info, "247", false))
            .arg(gum_style(&countdown, "247", false))
            .arg(gum_style(&celeb, "247", false))
            .status();
    } else {
        println!("{} {}", hd_emoji, hd.english());
        println!(
            "Baha'i: {} {} {}",
            next_hd.day(),
            next_hd.month().transliteration(),
            next_hd.year()
        );
        println!(
            "Gregorian: {} (starts at sunset)",
            greg_start.format("%Y-%m-%d %H:%M %Z")
        );
        println!("{} days from now", diff);
        println!("Celebrated at: {}", celeb_time);
    }
}

fn show_holy_days(tz: Tz, coords: Option<Coordinates>, fancy: bool) {
    let now = chrono::Utc::now().with_timezone(&tz);
    let badi = match LocalBadiDate::from_datetime(now, coords) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let year = badi.year();

    let title = format!("Holy Days — Year {} B.E.", year);
    if fancy {
        println!("{}", gum_style(&title, "214", true));
    } else {
        println!("{}", title);
    }
    println!("{}", "─".repeat(55));

    let all_holy_days: Vec<BahaiHolyDay> = vec![
        BahaiHolyDay::NawRuz,
        BahaiHolyDay::Ridvan1st,
        BahaiHolyDay::Ridvan9th,
        BahaiHolyDay::Ridvan12th,
        BahaiHolyDay::DeclarationOfTheBab,
        BahaiHolyDay::AscensionOfBahaullah,
        BahaiHolyDay::MartyrdomOfTheBab,
        BahaiHolyDay::BirthOfTheBab,
        BahaiHolyDay::BirthOfBahaullah,
        BahaiHolyDay::DayOfTheCovenant,
        BahaiHolyDay::AscensionOfAbdulBaha,
    ];

    let template = BadiDate::new(year, BadiMonth::Month(1), 1).unwrap();

    // Collect into a vec sorted by day_of_year so they display chronologically
    let mut entries: Vec<(u16, BahaiHolyDay)> = all_holy_days
        .into_iter()
        .map(|hd| (hd.day_of_year(year), hd))
        .collect();
    entries.sort_by_key(|(doy, _)| *doy);

    for (day_of_year, hd) in entries {
        let hd_emoji = holy_day_emoji(&hd);
        let work_status = if hd.work_suspended() {
            "work suspended"
        } else {
            "observance"
        };

        if let Ok(hd_date) = template.with_year_and_doy(year, day_of_year) {
            if let Ok(hd_local) =
                LocalBadiDate::new(hd_date.year(), hd_date.month(), hd_date.day(), tz, coords)
            {
                let greg = hd_local.start();
                let badi_str = format!("{} {}", hd_date.day(), hd_date.month().transliteration());
                let greg_str = greg.format("%Y-%m-%d").to_string();

                if fancy {
                    println!(
                        "{} {:<36} {:<18} {} ({})",
                        hd_emoji,
                        gum_style(&hd.english(), "214", false),
                        gum_style(&badi_str, "183", false),
                        gum_style(&greg_str, "247", false),
                        work_status,
                    );
                } else {
                    println!(
                        "{} {:<36} {:<18} {} ({})",
                        hd_emoji,
                        hd.english(),
                        badi_str,
                        greg_str,
                        work_status,
                    );
                }
            }
        }
    }
}

fn holy_day_emoji(holy_day: &BahaiHolyDay) -> &'static str {
    match holy_day {
        BahaiHolyDay::NawRuz => "🌺",
        BahaiHolyDay::Ridvan1st => "🌹",
        BahaiHolyDay::Ridvan9th => "🌹",
        BahaiHolyDay::Ridvan12th => "🌹",
        BahaiHolyDay::DeclarationOfTheBab => "📜",
        BahaiHolyDay::AscensionOfBahaullah => "⬆️",
        BahaiHolyDay::MartyrdomOfTheBab => "🕯️",
        BahaiHolyDay::BirthOfTheBab => "🎂",
        BahaiHolyDay::BirthOfBahaullah => "🎂",
        BahaiHolyDay::DayOfTheCovenant => "🤝",
        BahaiHolyDay::AscensionOfAbdulBaha => "🕊️",
    }
}

fn celebration_time(holy_day: &BahaiHolyDay) -> &'static str {
    match holy_day {
        BahaiHolyDay::NawRuz => "Sunset (start of the Badi day)",
        BahaiHolyDay::Ridvan1st => "Around 3:00 PM (before sunset)",
        BahaiHolyDay::Ridvan9th => "Around 3:00 PM (before sunset)",
        BahaiHolyDay::Ridvan12th => "Around 3:00 PM (before sunset)",
        BahaiHolyDay::DeclarationOfTheBab => "About 2 hours after sunset",
        BahaiHolyDay::AscensionOfBahaullah => "Around 3:00 AM",
        BahaiHolyDay::MartyrdomOfTheBab => "Solar noon (around 12:00 PM)",
        BahaiHolyDay::BirthOfTheBab => "Sunset (start of the Badi day)",
        BahaiHolyDay::BirthOfBahaullah => "Sunset (start of the Badi day)",
        BahaiHolyDay::DayOfTheCovenant => "During the day",
        BahaiHolyDay::AscensionOfAbdulBaha => "Around 1:00 AM",
    }
}
