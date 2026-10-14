//! Baha'i Date Converter CLI
//!
//! A simple tool to convert between Gregorian and Badi' (Baha'i) calendar dates.

use badi_date::{
    BadiDate, BadiDateLike, BadiMonth, BahaiHolyDay, Coordinates, FromDateTime,
    HolyDayProviding, LocalBadiDate, LocalBadiDateLike, ToDateTime,
};
use chrono::{Datelike, TimeZone, Weekday};
use chrono_tz::Tz;
use clap::{Parser, Subcommand};

/// Default location: Bahji
const DEFAULT_LATITUDE: f64 = 32.9434;
const DEFAULT_LONGITUDE: f64 = 35.0924;
const FALLBACK_TIMEZONE: &str = "Asia/Jerusalem";

/// A calendar name: transliteration, English meaning, Arabic and Persian script.
struct NameInfo {
    name: &'static str,
    meaning: &'static str,
    arabic: &'static str,
    persian: &'static str,
}

/// Which scripts to append to names (driven by --arabic / --persian / --arfa).
#[derive(Clone, Copy)]
struct NameFlags {
    arabic: bool,
    persian: bool,
}

/// --text / --text-all output selection.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TextMode {
    Off,
    /// Just the date: "9 Mashíyyat (Will) 183"
    Date,
    /// Date plus weekday, Vahid year, cycles, and annotations
    All,
}

impl NameFlags {
    /// "Mashíyyat (Will)" or, with flags, "Mashíyyat (Will · مشيّة)"
    fn format(&self, info: &NameInfo) -> String {
        let mut parts = vec![info.name];
        if !info.meaning.is_empty() {
            parts.push(info.meaning);
        }
        if self.arabic && !info.arabic.is_empty() {
            parts.push(info.arabic);
        }
        if self.persian && !info.persian.is_empty() && info.persian != info.arabic {
            parts.push(info.persian);
        }
        if parts.len() == 1 {
            return parts[0].to_string();
        }
        format!("{} ({})", parts[0], parts[1..].join(" · "))
    }

    /// Inline variant for embedding inside existing parens:
    /// "Mashíyyat — Will" or, with flags, "Mashíyyat — Will · مشيّة"
    fn format_inline(&self, info: &NameInfo) -> String {
        let mut parts = vec![info.name.to_string()];
        if !info.meaning.is_empty() {
            parts.push(format!("— {}", info.meaning));
        }
        if self.arabic && !info.arabic.is_empty() {
            parts.push(info.arabic.to_string());
        }
        if self.persian && !info.persian.is_empty() && info.persian != info.arabic {
            parts.push(info.persian.to_string());
        }
        parts.join(" ")
    }

    /// Script spellings for a holy day, in display order.
    fn holy_day_scripts(&self, holy_day: &BahaiHolyDay) -> Vec<&'static str> {
        let script = holy_day_script(holy_day);
        let mut parts = Vec::new();
        if self.arabic && !script.arabic.is_empty() {
            parts.push(script.arabic);
        }
        if self.persian && !script.persian.is_empty() && script.persian != script.arabic {
            parts.push(script.persian);
        }
        parts
    }

    /// "Naw-Rúz" or, with flags, "Naw-Rúz (عيد النَّيروز · عید نوروز)"
    fn format_holy_day(&self, holy_day: &BahaiHolyDay) -> String {
        let parts = self.holy_day_scripts(holy_day);
        if parts.is_empty() {
            holy_day.english()
        } else {
            format!("{} ({})", holy_day.english(), parts.join(" · "))
        }
    }
}

/// The 19 months (source: "Bahá'í Calendar Data - Overview.csv").
const MONTHS: [NameInfo; 19] = [
    NameInfo { name: "Bahá", meaning: "Splendor", arabic: "بهاء", persian: "بهاء" },
    NameInfo { name: "Jalál", meaning: "Glory", arabic: "جلال", persian: "جلال" },
    NameInfo { name: "Jamál", meaning: "Beauty", arabic: "جمال", persian: "جمال" },
    NameInfo { name: "'Azamat", meaning: "Grandeur", arabic: "عظمة", persian: "عظمت" },
    NameInfo { name: "Núr", meaning: "Light", arabic: "نور", persian: "نور" },
    NameInfo { name: "Rahmat", meaning: "Mercy", arabic: "رحمة", persian: "رحمت" },
    NameInfo { name: "Kalimát", meaning: "Words", arabic: "كلمات", persian: "کلمات" },
    NameInfo { name: "Kamál", meaning: "Perfection", arabic: "كمال", persian: "کمال" },
    NameInfo { name: "Asmá'", meaning: "Names", arabic: "أسماء", persian: "اسماء" },
    NameInfo { name: "'Izzat", meaning: "Might", arabic: "عزّة", persian: "عزّت" },
    NameInfo { name: "Mashíyyat", meaning: "Will", arabic: "مشيّة", persian: "مشیّت" },
    NameInfo { name: "'Ilm", meaning: "Knowledge", arabic: "علم", persian: "علم" },
    NameInfo { name: "Qudrat", meaning: "Power", arabic: "قدرة", persian: "قدرت" },
    NameInfo { name: "Qawl", meaning: "Speech", arabic: "قول", persian: "قول" },
    NameInfo { name: "Masá'il", meaning: "Questions", arabic: "مسائل", persian: "مسائل" },
    NameInfo { name: "Sharaf", meaning: "Honor", arabic: "شرف", persian: "شرف" },
    NameInfo { name: "Sultán", meaning: "Sovereignty", arabic: "سلطان", persian: "سلطان" },
    NameInfo { name: "Mulk", meaning: "Dominion", arabic: "ملك", persian: "ملک" },
    NameInfo { name: "'Alá'", meaning: "Loftiness", arabic: "علاء", persian: "علاء" },
];

/// Ayyám-i-Há (not in the CSV; spellings follow the corrected badiDate locales).
const AYYAM_I_HA: NameInfo = NameInfo {
    name: "Ayyám-i-Há",
    meaning: "The Days of Há",
    arabic: "أيّام الهاء",
    persian: "ایّام ها",
};

/// The 19 years of the Váḥid cycle (source: "Bahá'í Calendar Data - Overview.csv").
const VAHID_YEARS: [NameInfo; 19] = [
    NameInfo { name: "Alif", meaning: "A (The Letter)", arabic: "ألف", persian: "الف" },
    NameInfo { name: "Bá'", meaning: "B (The Letter)", arabic: "باء", persian: "باء" },
    NameInfo { name: "Ab", meaning: "Father", arabic: "أب", persian: "اب" },
    NameInfo { name: "Dál", meaning: "D (The Letter)", arabic: "دال", persian: "دال" },
    NameInfo { name: "Báb", meaning: "Gate", arabic: "باب", persian: "باب" },
    NameInfo { name: "Váv", meaning: "V (The Letter)", arabic: "واو", persian: "واو" },
    NameInfo { name: "Abad", meaning: "Eternity", arabic: "أبد", persian: "ابد" },
    NameInfo { name: "Jád", meaning: "Generosity", arabic: "جاد", persian: "جاد" },
    NameInfo { name: "Bahá", meaning: "Splendor", arabic: "بهاء", persian: "بهاء" },
    NameInfo { name: "Hubb", meaning: "Love", arabic: "حب", persian: "حبّ" },
    NameInfo { name: "Bahháj", meaning: "Delightful", arabic: "بهاج", persian: "بهّاج" },
    NameInfo { name: "Javáb", meaning: "Answer", arabic: "جواب", persian: "جواب" },
    NameInfo { name: "Ahad", meaning: "Single", arabic: "احد", persian: "احد" },
    NameInfo { name: "Vahháb", meaning: "Bountiful", arabic: "وهاب", persian: "وهّاب" },
    NameInfo { name: "Vidád", meaning: "Affection", arabic: "وداد", persian: "وداد" },
    NameInfo { name: "Badí'", meaning: "Beginning", arabic: "بديع", persian: "بدیع" },
    NameInfo { name: "Bahí", meaning: "Luminous", arabic: "بهي", persian: "بهی" },
    NameInfo { name: "Abhá", meaning: "Most Luminous", arabic: "ابهى", persian: "ابهی" },
    NameInfo { name: "Váhid", meaning: "Unity", arabic: "واحد", persian: "واحد" },
];

fn month_info(month: &BadiMonth) -> &'static NameInfo {
    match month {
        BadiMonth::Month(n) if (1..=19).contains(n) => &MONTHS[(*n - 1) as usize],
        _ => &AYYAM_I_HA,
    }
}

fn month_emoji(month: &BadiMonth) -> &'static str {
    match month {
        BadiMonth::AyyamIHa => "🎉",
        BadiMonth::Month(1) => "✨",
        BadiMonth::Month(2) => "🌟",
        BadiMonth::Month(3) => "🌸",
        BadiMonth::Month(4) => "🏛️",
        BadiMonth::Month(5) => "💡",
        BadiMonth::Month(6) => "🤲",
        BadiMonth::Month(7) => "📖",
        BadiMonth::Month(8) => "🎭",
        BadiMonth::Month(9) => "📛",
        BadiMonth::Month(10) => "💪",
        BadiMonth::Month(11) => "🎯",
        BadiMonth::Month(12) => "📚",
        BadiMonth::Month(13) => "⚡",
        BadiMonth::Month(14) => "💬",
        BadiMonth::Month(15) => "❓",
        BadiMonth::Month(16) => "🏅",
        BadiMonth::Month(17) => "👑",
        BadiMonth::Month(18) => "🌍",
        BadiMonth::Month(19) => "🕊️",
        BadiMonth::Month(_) => "📅",
    }
}

/// Weekday names; Saturday = Jalál (source: "Bahá'í Calendar Data - Overview.csv").
fn weekday_info(weekday: Weekday) -> NameInfo {
    match weekday {
        Weekday::Sat => NameInfo { name: "Jalál", meaning: "Glory", arabic: "جلال", persian: "جلال" },
        Weekday::Sun => NameInfo { name: "Jamál", meaning: "Beauty", arabic: "جمال", persian: "جمال" },
        Weekday::Mon => NameInfo { name: "Kamál", meaning: "Perfection", arabic: "كمال", persian: "کمال" },
        Weekday::Tue => NameInfo { name: "Fidál", meaning: "Grace", arabic: "فضال", persian: "فضال" },
        Weekday::Wed => NameInfo { name: "'Idál", meaning: "Justice", arabic: "عدال", persian: "عدال" },
        Weekday::Thu => NameInfo { name: "Istijlál", meaning: "Majesty", arabic: "استجلال", persian: "استجلال" },
        Weekday::Fri => NameInfo { name: "Istiqlál", meaning: "Independence", arabic: "استقلال", persian: "استقلال" },
    }
}

/// The Badi day runs sunset-to-sunset; its weekday name matches the Gregorian
/// weekday of the daylight portion, i.e. the day after the sunset that begins it.
fn badi_weekday(badi: &LocalBadiDate) -> Weekday {
    let start_date = badi.start().date_naive();
    start_date.succ_opt().unwrap_or(start_date).weekday()
}

/// Arabic and Persian script for a holy day (spellings from the corrected badiDate locales).
struct HolyDayScript {
    arabic: &'static str,
    persian: &'static str,
}

fn holy_day_script(holy_day: &BahaiHolyDay) -> HolyDayScript {
    match holy_day {
        BahaiHolyDay::NawRuz => HolyDayScript {
            arabic: "عيد النَّيروز",
            persian: "عید نوروز",
        },
        BahaiHolyDay::Ridvan1st => HolyDayScript {
            arabic: "اليوم الأول من عيد الرِّضوان",
            persian: "روز اوّل عید رضوان",
        },
        BahaiHolyDay::Ridvan9th => HolyDayScript {
            arabic: "اليوم التاسع من عيد الرِّضوان",
            persian: "روز نهم عید رضوان",
        },
        BahaiHolyDay::Ridvan12th => HolyDayScript {
            arabic: "اليوم الثاني عشر من عيد الرِّضوان",
            persian: "روز دوازدهم عید رضوان",
        },
        BahaiHolyDay::DeclarationOfTheBab => HolyDayScript {
            arabic: "يوم إعلان دعوة حضرة الباب",
            persian: "اظهار امر حضرت باب",
        },
        BahaiHolyDay::AscensionOfBahaullah => HolyDayScript {
            arabic: "يوم صعود حضرة بهاء الله",
            persian: "صعود حضرت بهاءالله",
        },
        BahaiHolyDay::MartyrdomOfTheBab => HolyDayScript {
            arabic: "يوم استشهاد حضرة الباب",
            persian: "شهادت حضرت باب",
        },
        BahaiHolyDay::BirthOfTheBab => HolyDayScript {
            arabic: "يوم ولادة حضرة الباب",
            persian: "تولّد حضرت باب",
        },
        BahaiHolyDay::BirthOfBahaullah => HolyDayScript {
            arabic: "يوم ولادة حضرة بهاء الله",
            persian: "تولّد حضرت بهاءالله",
        },
        BahaiHolyDay::DayOfTheCovenant => HolyDayScript {
            arabic: "يوم الميثاق",
            persian: "روز عهد و میثاق",
        },
        BahaiHolyDay::AscensionOfAbdulBaha => HolyDayScript {
            arabic: "يوم صعود حضرة عبد البهاء",
            persian: "صعود حضرت عبدالبهاء",
        },
    }
}

/// Name of the year within the current Váḥid cycle (1-based Badi year).
fn vahid_year_info(year: u16) -> &'static NameInfo {
    &VAHID_YEARS[(year.saturating_sub(1) % 19) as usize]
}

/// "Year 183 Javáb (Answer) · Vahid 10 · Kull-i-Shay 1"
fn year_line(year: u16, names: NameFlags) -> String {
    let vahid = ((year.saturating_sub(1)) / 19) + 1;
    let kull_i_shay = ((year.saturating_sub(1)) / 361) + 1;
    format!(
        "Year {} {} · Vahid {} · Kull-i-Shay {}",
        year,
        names.format(vahid_year_info(year)),
        vahid,
        kull_i_shay
    )
}

/// Holy day / Ayyám-i-Há / Feast annotation lines; emoji prefixes optional.
fn badi_annotations(badi: &LocalBadiDate, emoji: bool, names: NameFlags) -> Vec<String> {
    let month = badi.month();
    let mut out = Vec::new();

    if let Ok(badi_date) = BadiDate::new(badi.year(), month, badi.day()) {
        if let Some(hd) = badi_date.holy_day() {
            let prefix = if emoji {
                format!("{} ", holy_day_emoji(&hd))
            } else {
                String::new()
            };
            let work = if hd.work_suspended() {
                "work suspended"
            } else {
                "observance"
            };
            let mut bits = vec![work];
            bits.extend(names.holy_day_scripts(&hd));
            out.push(format!("{}{} ({})", prefix, hd.english(), bits.join(" · ")));
        }
    }

    if month == BadiMonth::AyyamIHa {
        let total = BadiMonth::AyyamIHa.number_of_days(badi.year());
        let prefix = if emoji { "🎉 " } else { "" };
        out.push(format!(
            "{}Ayyam-i-Ha Day {} of {}",
            prefix,
            badi.day(),
            total
        ));
    }

    if badi.is_feast() {
        let prefix = if emoji { "🍽️  " } else { "" };
        out.push(format!(
            "{}Nineteen Day Feast of {}",
            prefix,
            month_info(&month).name
        ));
    }

    out
}

/// One-line output for widgets: no line breaks, no emoji, no ANSI codes,
/// parts separated by " · ". `full` (--text-all) adds weekday, Vahid year,
/// cycles, and annotations; otherwise just the date (--text).
fn text_line(badi: &LocalBadiDate, names: NameFlags, full: bool) -> String {
    let date = format!(
        "{} {} {}",
        badi.day(),
        names.format(month_info(&badi.month())),
        badi.year()
    );
    if !full {
        return date;
    }

    let mut parts = vec![
        date,
        names.format(&weekday_info(badi_weekday(badi))),
        year_line(badi.year() as u16, names),
    ];
    parts.extend(badi_annotations(badi, false, names));
    parts.join(" · ")
}

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

    /// Include Arabic-script names of months, weekdays, Vahid years, and holy days
    #[arg(long, global = true)]
    arabic: bool,

    /// Include Persian-script names of months, weekdays, Vahid years, and holy days
    #[arg(long, global = true)]
    persian: bool,

    /// Include both Arabic- and Persian-script names
    #[arg(long, global = true)]
    arfa: bool,

    /// Output just the date as a single plain line (for widgets, Emacs, scripts)
    #[arg(long, global = true)]
    text: bool,

    /// Like --text, but includes weekday, Vahid year, and annotations
    #[arg(long, global = true, conflicts_with = "text")]
    text_all: bool,
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

    let names = NameFlags {
        arabic: cli.arabic || cli.arfa,
        persian: cli.persian || cli.arfa,
    };
    let text_mode = if cli.text_all {
        TextMode::All
    } else if cli.text {
        TextMode::Date
    } else {
        TextMode::Off
    };

    if let Some(entry) = cli.progress {
        show_progress(tz, coords.ok(), &entry, names);
        return;
    }

    if let Some(bucket) = cli.countdown {
        show_countdown(tz, coords.ok(), &bucket, names);
        return;
    }

    match cli.command.unwrap_or(Commands::Today) {
        Commands::Today => {
            if text_mode != TextMode::Off {
                show_text_today(tz, coords.ok(), text_mode, names);
            } else if cli.fancy {
                show_fancy_today(tz, coords.ok(), names);
            } else {
                show_today(tz, coords.ok(), names);
            }
        }
        Commands::ToBadi { year, month, day, hour, minute } => {
            greg_to_badi(year, month, day, hour.unwrap_or(12), minute.unwrap_or(0), tz, coords.ok(), cli.fancy, text_mode, names);
        }
        Commands::ToGreg { year, month, day } => {
            badi_to_greg(year, month, day, tz, coords.ok(), cli.fancy, text_mode, names);
        }
        Commands::NextHolyDay => {
            show_next_holy_day(tz, coords.ok(), cli.fancy, text_mode, names);
        }
        Commands::HolyDays => {
            show_holy_days(tz, coords.ok(), cli.fancy, names);
        }
    }
}

fn show_fancy_today(tz: Tz, coords: Option<Coordinates>, names: NameFlags) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => {
            show_fancy_badi(&badi, now, names);
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_fancy_badi(badi: &LocalBadiDate, dt: chrono::DateTime<Tz>, names: NameFlags) {
    let month = badi.month();
    let month_name = names.format(month_info(&month));
    let day = badi.day();
    let year = badi.year();
    let greg_date = dt.format("%Y-%m-%d").to_string();
    let greg_time = dt.format("%H:%M %Z").to_string();
    let tz_name = badi.timezone().name();

    let header = format!("{} {}", month_emoji(&month), month_name);
    let subheader = format!("Day {}  •  Year {}", day, year);
    let weekday_line = format!("Weekday: {}", names.format(&weekday_info(badi_weekday(badi))));
    let year_line = year_line(year as u16, names);
    let greg_info = format!("{} {}", greg_date, greg_time);
    let tz_info = format!("🕐 {}", tz_name);

    let mut args: Vec<String> = vec![
        gum_style(&header, "141", true),
        gum_style(&subheader, "183", false),
        gum_style(&weekday_line, "183", false),
        gum_style(&year_line, "183", false),
        String::new(),
        gum_style(&greg_info, "247", false),
        gum_style(&tz_info, "247", false),
    ];

    // Build annotation lines for holy day / Ayyam-i-Ha / feast
    let annotations = badi_annotations(badi, true, names);

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

fn show_today(tz: Tz, coords: Option<Coordinates>, names: NameFlags) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => print_badi_date(&badi, now, names, false),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_text_today(tz: Tz, coords: Option<Coordinates>, mode: TextMode, names: NameFlags) {
    let now = chrono::Utc::now().with_timezone(&tz);
    match LocalBadiDate::from_datetime(now, coords) {
        Ok(badi) => println!("{}", text_line(&badi, names, mode == TextMode::All)),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn show_progress(tz: Tz, coords: Option<Coordinates>, entry: &str, names: NameFlags) {
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
        print_fancy_progress_bar(
            "Month",
            month_progress,
            &names.format_inline(month_info(&badi.month())),
        );
        print_fancy_progress_bar(
            "Year",
            year_progress,
            &format!(
                "Year {} — {}",
                badi.year(),
                names.format_inline(vahid_year_info(badi.year() as u16))
            ),
        );
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
                                    names.format_holy_day(&hd),
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
            "month" => print_plain_progress_bar(
                "Month",
                month_progress,
                &names.format_inline(month_info(&badi.month())),
            ),
            "year" => print_plain_progress_bar(
                "Year",
                year_progress,
                &format!(
                    "Year {} — {}",
                    badi.year(),
                    names.format_inline(vahid_year_info(badi.year() as u16))
                ),
            ),
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

fn show_countdown(tz: Tz, coords: Option<Coordinates>, bucket: &str, names: NameFlags) {
    let now = chrono::Utc::now().with_timezone(&tz);
    let badi = match LocalBadiDate::from_datetime(now, coords) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    if bucket == "all" {
        show_countdown_all(&badi, now, tz, coords, names);
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
    names: NameFlags,
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
                        names.format_holy_day(&hd)
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
    text_mode: TextMode,
    names: NameFlags,
) {
    match tz.with_ymd_and_hms(year, month, day, hour, minute, 0).single() {
        Some(dt) => match LocalBadiDate::from_datetime(dt, coords) {
            Ok(badi) => {
                if text_mode != TextMode::Off {
                    println!("{}", text_line(&badi, names, text_mode == TextMode::All));
                } else if fancy {
                    show_fancy_badi(&badi, dt, names);
                } else {
                    print_badi_date(&badi, dt, names, false);
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
    text_mode: TextMode,
    names: NameFlags,
) {
    let badi_month = if month == 0 {
        BadiMonth::AyyamIHa
    } else {
        BadiMonth::Month(month)
    };

    match LocalBadiDate::new(year as u8, badi_month, day as u16, tz, coords) {
        Ok(badi) => {
            let start = badi.start();
            if text_mode != TextMode::Off {
                println!(
                    "{} · Gregorian: {} (starts at sunset)",
                    text_line(&badi, names, text_mode == TextMode::All),
                    start.format("%Y-%m-%d %H:%M %Z")
                );
            } else if fancy {
                show_fancy_badi(&badi, start, names);
            } else {
                print_badi_date(&badi, start, names, true);
            }
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn print_badi_date(badi: &LocalBadiDate, dt: chrono::DateTime<Tz>, names: NameFlags, sunset_note: bool) {
    let month = badi.month();
    let month_name = names.format(month_info(&month));

    println!("{} {} {}", badi.day(), month_name, badi.year());
    println!(
        "Weekday: {}",
        names.format(&weekday_info(badi_weekday(badi)))
    );
    println!("{}", year_line(badi.year() as u16, names));
    println!(
        "Gregorian: {} {}{}",
        dt.format("%Y-%m-%d"),
        dt.format("%H:%M %Z"),
        if sunset_note { " (starts at sunset)" } else { "" }
    );
    println!("Timezone: {}", badi.timezone().name());

    for ann in badi_annotations(badi, true, names) {
        println!("{}", ann);
    }
}

fn show_next_holy_day(
    tz: Tz,
    coords: Option<Coordinates>,
    fancy: bool,
    text_mode: TextMode,
    names: NameFlags,
) {
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

    if text_mode != TextMode::Off {
        let greg = format!(
            "{} {}",
            greg_start.format("%Y-%m-%d"),
            greg_start.format("%H:%M %Z")
        );
        let date_part = format!(
            "{} · {} {} {} · {} · {} days from now",
            names.format_holy_day(&hd),
            next_hd.day(),
            names.format(month_info(&next_hd.month())),
            next_hd.year(),
            greg,
            diff
        );
        if text_mode == TextMode::All {
            println!("{} · Celebrated at: {}", date_part, celeb_time);
        } else {
            println!("{}", date_part);
        }
    } else if fancy {
        let header = format!("{} {}", hd_emoji, names.format_holy_day(&hd));
        let badi_info = format!(
            "{} {} {}",
            next_hd.day(),
            names.format(month_info(&next_hd.month())),
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
        println!("{} {}", hd_emoji, names.format_holy_day(&hd));
        println!(
            "Baha'i: {} {} {}",
            next_hd.day(),
            names.format(month_info(&next_hd.month())),
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

fn show_holy_days(tz: Tz, coords: Option<Coordinates>, fancy: bool, names: NameFlags) {
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

    // Script flags widen the holy day column rather than breaking its alignment.
    let name_cells: Vec<String> = entries
        .iter()
        .map(|(_, hd)| names.format_holy_day(hd))
        .collect();
    let name_width = name_cells
        .iter()
        .map(|cell| cell.chars().count())
        .max()
        .unwrap_or(36)
        .max(36);

    for ((day_of_year, hd), name_cell) in entries.into_iter().zip(name_cells) {
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
                let badi_str = format!("{} {}", hd_date.day(), month_info(&hd_date.month()).name);
                let greg_str = greg.format("%Y-%m-%d").to_string();

                if fancy {
                    println!(
                        "{} {:<width$} {:<18} {} ({})",
                        hd_emoji,
                        gum_style(&name_cell, "214", false),
                        gum_style(&badi_str, "183", false),
                        gum_style(&greg_str, "247", false),
                        work_status,
                        width = name_width,
                    );
                } else {
                    println!(
                        "{} {:<width$} {:<18} {} ({})",
                        hd_emoji,
                        name_cell,
                        badi_str,
                        greg_str,
                        work_status,
                        width = name_width,
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
