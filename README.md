![A crummy, quick logo for the Bahai-Date-Rust project.](https://m1s5.c20.e2-5.dev/files/images/bahai-date-rust-logo.svg)

# Baha'i Date Converter

A command-line tool to convert between Gregorian and Badí' (Bahá'í) calendar dates, watch the Baha'i year progress, and count down to feasts and holy days. Works entirely offline.

Dates print with month, weekday, and Váḥid-year names and their English meanings — plus the original Arabic/Persian scripts on request (`--arfa`) — or as a bare one-liner for widgets and scripts (`--text`).

## Installation

### Build from Source (recommended)

The pre-built binaries in the [releases](https://forge.mitch.science/mitcheffendi/bahai-date-rust/releases) are assembled by hand — there is no CI/CD pipeline producing them yet, so they may lag behind the source. Building from source is the reliable path:

```bash
git clone https://forge.mitch.science/mitcheffendi/bahai-date-rust
cd bahai-date-rust
cargo install --path .
```

You need a [Rust](https://rustup.rs) toolchain (stable). `./install.sh` runs the same `cargo install --path .` for you.

To pick up changes later, pull and rebuild:

```bash
git pull
cargo install --path . --force
```

`--force` overwrites the older binary in `~/.cargo/bin`. Run `cargo update` first if you also want refreshed dependencies (the committed `Cargo.lock` pins working versions).

### Pre-built Binaries

Manually built, not produced by CI — use only if you can't install Rust. Grab them from the [releases](https://forge.mitch.science/mitcheffendi/bahai-date-rust/releases) page and check the commit they were cut from.

## Quickstart

```bash
# Today's date (no arguments needed)
$ bahai-date
9 Mashíyyat (Will) 183
Weekday: Kamál (Perfection)
Year 183 Javáb (Answer) · Vahid 10 · Kull-i-Shay 1
Gregorian: 2026-10-04 17:29 EDT
Timezone: America/New_York

# Add the Arabic and Persian scripts (--arabic, --persian, or --arfa for both)
$ bahai-date --arfa
9 Mashíyyat (Will · مشيئة · مشيت) 183
Weekday: Kamál (Perfection · كمال · کمال)
Year 183 Javáb (Answer · جواب) · Vahid 10 · Kull-i-Shay 1
Gregorian: 2026-10-04 17:29 EDT
Timezone: America/New_York

# Single line for widgets, Emacs, and scripts (--text = just the date;
# --text-all adds weekday, Vahid year, and cycles)
$ bahai-date --text
9 Mashíyyat (Will) 183

$ bahai-date --text-all
9 Mashíyyat (Will) 183 · Kamál (Perfection) · Year 183 Javáb (Answer) · Vahid 10 · Kull-i-Shay 1

# Days until the next Nineteen Day Feast
$ bahai-date --countdown feast

# Full-screen progress view of day / month / year / Vahid / Epoch
$ bahai-date --progress

# Pretty output everywhere (requires gum, see below)
$ bahai-date --fancy
```

`bahai-date` with no subcommand is equivalent to `bahai-date today`.

## Feature Flags

All flags below are *global*: they work with any subcommand, before or after it. The groups, simplest first: script display (`--arabic`/`--persian`/`--arfa`), one-line output (`--text`/`--text-all`), pretty-printing (`--fancy`/`--progress`), and countdowns (`--countdown`).

### Name Scripts

Month, weekday, and Váḥid-year names always print with their English meanings; these flags add the original scripts:

| Flag | What it does |
|------|--------------|
| `--arabic` | Include the Arabic-script name |
| `--persian` | Include the Persian-script name |
| `--arfa` | Include both Arabic and Persian scripts (identical spellings are shown once) |

```bash
$ bahai-date to-greg -Y 181 -M 1 -D 1 --arfa
1 Bahá (Splendor · بهاء) 181
Weekday: 'Idál (Justice · عدال)
Year 181 Hubb (Love · حب) · Vahid 10 · Kull-i-Shay 1
...
```

> The `holy-days` table keeps fixed-width columns, so it always prints plain month names regardless of these flags.

### One-Line Output

| Flag | What it does |
|------|--------------|
| `--text` | Print just the date as a single plain line — no line breaks, no box, no emoji, no color — for widgets, Emacs, waybar, and other command consumers |
| `--text-all` | Same single line, plus weekday, Váḥid year, cycle position, and any annotations |

For daily glance value, `--text` gives what a Gregorian calendar would — the date and nothing else. `--text-all` is the whole picture in one line. The two flags are mutually exclusive.

```bash
$ bahai-date --text
9 Mashíyyat (Will) 183

$ bahai-date --text-all
9 Mashíyyat (Will) 183 · Kamál (Perfection) · Year 183 Javáb (Answer) · Vahid 10 · Kull-i-Shay 1

$ bahai-date to-greg -Y 181 -M 1 -D 1 -t Asia/Jerusalem --text
1 Bahá (Splendor) 181 · Gregorian: 2024-03-19 17:50 IST (starts at sunset)

$ bahai-date next-holy-day --text
Birth of the Báb · 7 Qudrat (Power) 183 · 2026-11-09 09:42 EST · 36 days from now
```

Both override `--fancy`; `--progress` still wins if given. They combine freely with `--arabic`, `--persian`, and `--arfa`.

### Pretty-Printing (TUI)

| Flag | What it does |
|------|--------------|
| `-f, --fancy` | Emoji + color output for `today`, `to-badi`, `to-greg`, `next-holy-day`, and `holy-days` |
| `-p, --progress` | Full-screen progress dashboard (see below) |

**`--fancy`** renders the date in a rounded box with a month emoji, and highlights holy days, Ayyám-i-Há, and Feast days; `next-holy-day` gets a boxed card with the countdown and celebration time:

```bash
$ bahai-date --fancy
$ bahai-date next-holy-day --fancy
```

**`--progress`** (bare, or `-p all`) clears the screen and shows a Neofetch-style dashboard: progress bars for the Baha'i day, month, year, Vahid, and Kull-i-Shay (Epoch), plus the next holy day. It waits for <kbd>Enter</kbd> before exiting, so it's meant for terminals, not scripts.

> Both use [Charm's `gum`](https://github.com/charmbracelet/gum) when available. If you don't have `gum` installed, `--fancy` and the `--progress` dashboard won't work; the plain output and the single-entry progress bars below always work.

### Countdowns and Progress Views

| Flag | What it does |
|------|--------------|
| `-c, --countdown [BUCKET]` (alias `--cd`) | Time remaining until the next day / feast / holy day / Naw-Rúz / Vahid |
| `-p, --progress ENTRY` | A single one-line progress bar for a calendar span |

**Countdown buckets**, simplest first. A single bucket prints exactly one plain line — a stable `NN days` or `H.H hours` value you can pipe or embed anywhere:

```bash
$ bahai-date --countdown day        # hours (or minutes) until the next Baha'i day begins at sunset
$ bahai-date --countdown feast      # days until the next Nineteen Day Feast (alias: month)
$ bahai-date --countdown holy-day   # days until the next Holy Day (alias: holyday)
$ bahai-date --countdown year       # days until Naw-Rúz (aliases: nawruz, naw-ruz)
$ bahai-date --countdown vahid      # days until the next Vahid begins
```

Bare `-c` (or `-c all`) prints an annotated summary of all of the above — including days until Ayyám-i-Há, years until the next Vahid, and Vahids until the next Kull-i-Shay.

Handy in shells, e.g.:

```bash
echo "Next Feast in $(bahai-date --countdown feast) days"
```

**Progress entries** print a single ASCII progress bar (`####---- 42.3% (Day 8)`) for the given span — script-friendly, no `gum` needed:

```bash
$ bahai-date --progress day      # position within the current 19-day month
$ bahai-date --progress month    # month within the year
$ bahai-date --progress year     # year since Naw-Rúz
$ bahai-date --progress vahid    # year within the current 19-year Vahid
$ bahai-date --progress epoch    # Vahid within the current 361-year Kull-i-Shay
                                 #   (aliases: kull-i-shay, kullishay)
```

If both `-p` and `-c` are given, the progress view wins.

### Programmatic Use

Plain (non-`--fancy`) output is line-oriented and stable, suitable for scripts: the Baha'i date, the weekday name, the year with its position in the Váḥid and Kull-i-Shay cycles, the Gregorian date/time, and the timezone — followed by any annotations (holy day, Ayyám-i-Há day, Feast). For a strict one-line string (widgets, Emacs), use [`--text`](#one-line-output).

| Command | Purpose |
|---------|---------|
| `bahai-date today` | Today's Baha'i date (the default when no subcommand is given) |
| `bahai-date to-badi` | Convert a Gregorian date to Baha'i |
| `bahai-date to-greg` | Convert a Baha'i date to Gregorian |
| `bahai-date next-holy-day` | Next upcoming holy day: name, dates, days away, celebration time |
| `bahai-date holy-days` | List all holy days for the current Baha'i year |

Global options:

| Option | Description |
|--------|-------------|
| `-t, --timezone <TIMEZONE>` | IANA timezone, e.g. `America/New_York` (spaces are OK: `America/New York`). Default: system timezone, falling back to `Asia/Jerusalem` |
| `-l, --lat <LAT>` | Latitude for sunset calculation. Default: Bahjí (32.9434) |
| `-L, --lon <LON>` | Longitude for sunset calculation. Default: Bahjí (35.0924) |
| `--arabic` | Include Arabic-script names (month, weekday, Váḥid year) |
| `--persian` | Include Persian-script names |
| `--arfa` | Include both Arabic- and Persian-script names |
| `--text` | Single-line output: just the date — no line breaks, no emoji, no color |
| `--text-all` | Single-line output with weekday, Váḥid year, cycles, and annotations |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

#### `to-badi`

Examples pin `-t Asia/Jerusalem` so the output is reproducible anywhere; omit it to use your system timezone.

```bash
$ bahai-date to-badi -Y 2017 -M 12 -D 5 -t Asia/Jerusalem
14 Qawl (Speech) 174
Weekday: Fidál (Grace)
Year 174 Ab (Father) · Vahid 10 · Kull-i-Shay 1
Gregorian: 2017-12-05 12:00 IST
Timezone: Asia/Jerusalem

# The time of day matters: days start at sunset, so 18:30 on the day
# after Naw-Ruz sunset is already the 2nd day of the year:
$ bahai-date to-badi -Y 2024 -M 3 -D 20 -H 18 -I 30 -t Asia/Jerusalem
2 Bahá (Splendor) 181
Weekday: Istijlál (Majesty)
Year 181 Hubb (Love) · Vahid 10 · Kull-i-Shay 1
Gregorian: 2024-03-20 18:30 IST
Timezone: Asia/Jerusalem
```

| Option | Description |
|--------|-------------|
| `-Y, --year <YEAR>` | Gregorian year (e.g., 2024) |
| `-M, --month <MONTH>` | Month (1-12) |
| `-D, --day <DAY>` | Day (1-31) |
| `-H, --hour <HOUR>` | Hour (0-23, optional, default: 12) |
| `-I, --minute <MINUTE>` | Minute (0-59, optional, default: 0) |

#### `to-greg`

```bash
$ bahai-date to-greg -Y 181 -M 1 -D 1 -t Asia/Jerusalem
1 Bahá (Splendor) 181
Weekday: 'Idál (Justice)
Year 181 Hubb (Love) · Vahid 10 · Kull-i-Shay 1
Gregorian: 2024-03-19 17:50 IST (starts at sunset)
Timezone: Asia/Jerusalem
🌺 Naw-Rúz (work suspended)
Nineteen Day Feast of Bahá
```

| Option | Description |
|--------|-------------|
| `-Y, --year <YEAR>` | Baha'i year (e.g., 181) |
| `-M, --month <MONTH>` | Month (0 = Ayyám-i-Há, 1-19) |
| `-D, --day <DAY>` | Day (1-19) |

## The Baha'i Calendar

The Badí' calendar consists of:
- **19 months** of 19 days each
- **Ayyám-i-Há** (intercalary days): 4-5 days between months 18 and 19
- **New Year (Naw-Rúz)**: Falls on March 20-21

### Month Names

| # | Name | Meaning | Arabic Script | Persian Script |
|---|------|---------|---------------|----------------|
| 1 | Bahá | Splendor | بهاء | بهاء |
| 2 | Jalál | Glory | جلال | جلال |
| 3 | Jamál | Beauty | جمال | جمال |
| 4 | 'Azamat | Grandeur | عظمة | عظمت |
| 5 | Núr | Light | نور | نور |
| 6 | Rahmat | Mercy | رحمة | رحمت |
| 7 | Kalimát | Words | كلمات | کلمات |
| 8 | Kamál | Perfection | كمال | کمال |
| 9 | Asmá' | Names | اسماء | اسماء |
| 10 | 'Izzat | Might | عزة | عزت |
| 11 | Mashíyyat | Will | مشيئة | مشيت |
| 12 | 'Ilm | Knowledge | علم | علم |
| 13 | Qudrat | Power | قدرة | قدرت |
| 14 | Qawl | Speech | قول | قول |
| 15 | Masá'il | Questions | مسائل | مسائل |
| 16 | Sharaf | Honor | شرف | شرف |
| 17 | Sultán | Sovereignty | سلطان | سلطان |
| 18 | Mulk | Dominion | ملك | ملک |
| 19 | 'Alá' | Loftiness | علاء | علاء |

### Weekday Names

The Badi day starts at sunset, so each weekday begins the evening before its Gregorian counterpart (Jalál begins at Friday sunset).

| English Day | Name | Meaning | Arabic Script | Persian Script |
|-------------|------|---------|---------------|----------------|
| Saturday | Jalál | Glory | جلال | جلال |
| Sunday | Jamál | Beauty | جمال | جمال |
| Monday | Kamál | Perfection | كمال | کمال |
| Tuesday | Fidál | Grace | فضال | فضال |
| Wednesday | 'Idál | Justice | عدال | عدال |
| Thursday | Istijlál | Majesty | استجلال | استجلال |
| Friday | Istiqlál | Independence | استقلال | استقلال |

### Váḥid Cycle Years

Each 19-year Váḥid cycle names its years; 19 Váḥids make up a Kull-i-Shay' (361 years). Year 1 B.E. began the first Váḥid of the first Kull-i-Shay' in 1844.

| Year # | Name | Meaning | Arabic Script | Persian Script |
|--------|------|---------|---------------|----------------|
| 1 | Alif | A (The Letter) | الف | الف |
| 2 | Bá' | B (The Letter) | باء | باء |
| 3 | Ab | Father | اب | اب |
| 4 | Dál | D (The Letter) | دال | دال |
| 5 | Báb | Gate | باب | باب |
| 6 | Váv | V (The Letter) | واو | واو |
| 7 | Abad | Eternity | ابد | ابد |
| 8 | Jád | Generosity | جاد | جاد |
| 9 | Bahá | Splendor | بهاء | بهاء |
| 10 | Hubb | Love | حب | حب |
| 11 | Bahháj | Delightful | بهاج | بهاج |
| 12 | Javáb | Answer | جواب | جواب |
| 13 | Ahad | Single | احد | احد |
| 14 | Vahháb | Bountiful | وهاب | وهاب |
| 15 | Vidád | Affection | وداد | وداد |
| 16 | Badí' | Beginning | بديع | بديع |
| 17 | Bahí | Luminous | بهي | بهي |
| 18 | Abhá | Most Luminous | ابهى | ابهى |
| 19 | Váhid | Unity | واحد | واحد |

## Notes

- The Baha'i day starts at **sunset**, not midnight
- Default location is **Bahjí** (32.9434°N, 35.0924°E) for sunset calculations
- Use `-l` and `-L` options to specify custom coordinates for your location

## License

ISC
