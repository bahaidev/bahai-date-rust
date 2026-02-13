# Baha'i Date Converter

A simple command-line tool to convert between Gregorian and Badí' (Bahá'í) calendar dates. Works entirely offline.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/badi-date-rust.git
cd badi-date-rust
cargo install --path .
```

### Pre-built Binaries

Download from [releases](https://github.com/yourusername/badi-date-rust/releases).

## Usage

### Show Today's Date

```bash
$ bahai-date
8 Mulk 182
Gregorian: 2026-02-13 04:17 IST
Timezone: Asia/Jerusalem
```

### Convert Gregorian to Baha'i

```bash
$ bahai-date to-badi -Y 2017 -M 12 -D 5
14 Qawl 174
Gregorian: 2017-12-05 12:00 IST
Timezone: Asia/Jerusalem

# With time (affects day based on sunset)
$ bahai-date to-badi -Y 2024 -M 3 -D 20 -H 18 -I 30
1 Naw-Ruz 181
Gregorian: 2024-03-20 18:30 IST
Timezone: Asia/Jerusalem
```

### Convert Baha'i to Gregorian

```bash
$ bahai-date to-greg -Y 181 -M 1 -D 1
Baha'i: 1 Bahá 181
Gregorian: 2024-03-20 16:32 IST (starts at sunset)
```

### Custom Timezone

```bash
$ bahai-date -t "America/New_York"
8 Mulk 182
Gregorian: 2026-02-12 21:17 EST
Timezone: America/New_York

$ bahai-date to-badi -Y 2024 -M 3 -D 20 -t "Europe/London"
1 Naw-Ruz 181
Gregorian: 2024-03-20 12:00 GMT
Timezone: Europe/London
```

## Options

| Option | Description |
|--------|-------------|
| `-t, --timezone <TIMEZONE>` | Timezone (e.g., "America/New_York", "Asia/Jerusalem") |
| `-l, --lat <LAT>` | Latitude for sunset calculation |
| `-L, --lon <LON>` | Longitude for sunset calculation |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

### `to-badi` Command

| Option | Description |
|--------|-------------|
| `-Y, --year <YEAR>` | Gregorian year (e.g., 2024) |
| `-M, --month <MONTH>` | Month (1-12) |
| `-D, --day <DAY>` | Day (1-31) |
| `-H, --hour <HOUR>` | Hour (0-23, optional, default: 12) |
| `-I, --minute <MINUTE>` | Minute (0-59, optional, default: 0) |

### `to-greg` Command

| Option | Description |
|--------|-------------|
| `-Y, --year <YEAR>` | Baha'i year (e.g., 181) |
| `-M, --month <MONTH>` | Month (0 = Ayyam-i-Há, 1-19) |
| `-D, --day <DAY>` | Day (1-19) |

## The Baha'i Calendar

The Badí' calendar consists of:
- **19 months** of 19 days each
- **Ayyam-i-Há** (intercalary days): 4-5 days between months 18 and 19
- **New Year (Naw-Rúz)**: Falls on March 20-21

### Month Names

| # | Name | Meaning |
|---|------|---------|
| 1 | Bahá | Splendor |
| 2 | Jalál | Glory |
| 3 | Jamál | Beauty |
| 4 | 'Azamat | Grandeur |
| 5 | Núr | Light |
| 6 | Rahmat | Mercy |
| 7 | Kalimát | Words |
| 8 | Kamál | Perfection |
| 9 | Asmá' | Names |
| 10 | 'Izzat | Might |
| 11 | Mashíyyat | Will |
| 12 | 'Ilm | Knowledge |
| 13 | Qudrat | Power |
| 14 | Qawl | Speech |
| 15 | Masá'il | Questions |
| 16 | Sharaf | Honor |
| 17 | Sultán | Sovereignty |
| 18 | Mulk | Dominion |
| 19 | 'Alá' | Loftiness |

## Notes

- The Baha'i day starts at **sunset**, not midnight
- Default location is **Bahjí** (32.9434°N, 35.0924°E) for sunset calculations
- Use `-l` and `-L` options to specify custom coordinates for your location

## License

ISC
