#!/bin/bash

# Get the output
output=$(/Users/mitch/.cargo/bin/bahai-date --timezone "America/New_York")

# Parse the values
bahai_date=$(echo "$output" | head -1)
gregorian=$(echo "$output" | grep "^Gregorian:" | sed 's/Gregorian: //')
timezone=$(echo "$output" | grep "^Timezone:" | sed 's/Timezone: //')

# Extract month name (second word)
month=$(echo "$bahai_date" | awk '{print $2}')

# Get emoji for the month
case "$month" in
    "Bahá")    emoji="✨" ;;
    "Jalál")   emoji="🌟" ;;
    "Jamál")   emoji="🌸" ;;
    "Azamat")  emoji="🏛️" ;;
    "Núr")     emoji="💡" ;;
    "Rahmat")  emoji="🤲" ;;
    "Kalimát") emoji="📖" ;;
    "Kamál")   emoji="🎭" ;;
    "Asmá")    emoji="📛" ;;
    "Izzat")   emoji="💪" ;;
    "Mashíyyat") emoji="🎯" ;;
    "Ilm")     emoji="📚" ;;
    "Qudrat")  emoji="⚡" ;;
    "Qawl")    emoji="💬" ;;
    "Masá'il") emoji="❓" ;;
    "Sharaf")  emoji="🏅" ;;
    "Sultán")  emoji="👑" ;;
    "Mulk")    emoji="🌍" ;;
    "Alá")     emoji="🕊️" ;;
    *)         emoji="📅" ;;
esac

# Extract day and year
day=$(echo "$bahai_date" | awk '{print $1}')
year=$(echo "$bahai_date" | awk '{print $3}')

# Display beautifully with gum
gum style \
    --border rounded \
    --border-foreground 99 \
    --padding "1 2" \
    --margin 0 \
    "$(gum style --foreground 141 --bold "$emoji $month")" \
    "$(gum style --foreground 183 "Day $day  •  Year $year")" \
    "" \
    "$(gum style --foreground 247 "📅 $gregorian")" \
    "$(gum style --foreground 247 "🕐 $timezone")"
