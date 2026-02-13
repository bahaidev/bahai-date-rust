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

    match cli.command.unwrap_or(Commands::Today) {
        Commands::Today => show_today(tz, coords.ok()),
        Commands::ToBadi { year, month, day, hour, minute } => {
            greg_to_badi(year, month, day, hour.unwrap_or(12), minute.unwrap_or(0), tz, coords.ok());
        }
        Commands::ToGreg { year, month, day } => {
            badi_to_greg(year, month, day, tz, coords.ok());
        }
    }
}

fn show_today(tz: Tz, coords: Option<Coordinates>) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => print_badi_date(&badi, now),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn greg_to_badi(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: Tz, coords: Option<Coordinates>) {
    match tz.with_ymd_and_hms(year, month, day, hour, minute, 0).single() {
        Some(dt) => match LocalBadiDate::from_datetime(dt, coords) {
            Ok(badi) => print_badi_date(&badi, dt),
            Err(e) => eprintln!("Error: {:?}", e),
        },
        None => eprintln!("Error: Invalid date"),
    }
}

fn badi_to_greg(year: u16, month: u8, day: u8, tz: Tz, coords: Option<Coordinates>) {
    let badi_month = if month == 0 {
        BadiMonth::AyyamIHa
    } else {
        BadiMonth::Month(month)
    };

    match LocalBadiDate::new(year as u8, badi_month, day as u16, tz, coords) {
        Ok(badi) => {
            let start = badi.start();
            println!("Baha'i: {} {} {}", day, badi_month.transliteration(), year);
            println!("Gregorian: {} (starts at sunset)", start.format("%Y-%m-%d %H:%M %Z"));
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
