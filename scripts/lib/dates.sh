#!/bin/sh
# ==============================================================================
# scripts/lib/dates.sh
#
# Shared portable date utilities for maintenance scripts.
# Provides deterministic, leap-year-aware date conversions.
# No dependence on platform-specific date +%s or date -d.
# ==============================================================================

# Check if a year is a leap year
# Returns 0 (success) if leap year, 1 otherwise
is_leap_year() {
    __year="$1"
    if [ $((__year % 4)) -ne 0 ]; then
        unset __year
        return 1  # Not divisible by 4 → not leap year
    elif [ $((__year % 100)) -ne 0 ]; then
        unset __year
        return 0  # Divisible by 4, not by 100 → leap year
    elif [ $((__year % 400)) -eq 0 ]; then
        unset __year
        return 0  # Divisible by 400 → leap year
    else
        unset __year
        return 1  # Divisible by 100 but not 400 → not leap year
    fi
}

# Get days in a month, accounting for leap years
# Usage: days_in_month <month> <year>
days_in_month() {
    __month="$1"
    __year="$2"

    case "$__month" in
        1|3|5|7|8|10|12) echo 31 ;;
        4|6|9|11) echo 30 ;;
        2)
            if is_leap_year "$__year"; then
                echo 29
            else
                echo 28
            fi
            ;;
        *) unset __month __year; return 1 ;;
    esac
    unset __month __year
}

# Convert ISO 8601 date (YYYY-MM-DD) to seconds since epoch (portable)
# Uses shell arithmetic on date components; avoids platform-specific date flags.
date_to_epoch() {
    date_str="$1"

    # Parse YYYY-MM-DD
    year="${date_str%%-*}"
    rest="${date_str#*-}"
    month="${rest%%-*}"
    day="${rest#*-}"

    # Strip all leading zeros to avoid octal interpretation in arithmetic.
    while [ "${year#0}" != "$year" ]; do year="${year#0}"; done
    while [ "${month#0}" != "$month" ]; do month="${month#0}"; done
    while [ "${day#0}" != "$day" ]; do day="${day#0}"; done

    # Handle case where stripping leading 0 leaves empty (e.g., "0" → "")
    [ -z "$year" ] && year=0
    [ -z "$month" ] && month=0
    [ -z "$day" ] && day=0

    # Validate ranges
    if ! { [ "$month" -ge 1 ] && [ "$month" -le 12 ]; } || ! { [ "$day" -ge 1 ] && [ "$day" -le 31 ]; }; then
        unset rest days y m
        return 1
    fi

    # Assert year is >= 1970 to prevent pre-epoch calculation bugs
    if [ "$year" -lt 1970 ]; then
        unset rest days y m
        return 1
    fi

    # Use shell arithmetic to calculate epoch
    # Count days from 1970-01-01 to the given date
    days=0

    # Count days for complete years
    y=1970
    while [ "$y" -lt "$year" ]; do
        if is_leap_year "$y"; then
            days=$((days + 366))
        else
            days=$((days + 365))
        fi
        y=$((y + 1))
    done

    # Count days for months in the target year
    m=1
    while [ "$m" -lt "$month" ]; do
        days=$((days + $(days_in_month "$m" "$year")))
        m=$((m + 1))
    done

    # Add days in the target month
    days=$((days + day))

    # Convert to seconds
    result=$((days * 86400))
    echo "$result"
    unset rest days y m result
}

# Calculate days between two ISO 8601 dates
# Usage: days_between <date1> <date2>
# Returns positive if date2 > date1, negative if date2 < date1
days_between() {
    date1="$1"
    date2="$2"

    if ! epoch1=$(date_to_epoch "$date1"); then
        unset epoch1 diff
        return 1
    fi
    if ! epoch2=$(date_to_epoch "$date2"); then
        unset epoch1 epoch2 diff
        return 1
    fi

    diff=$((epoch2 - epoch1))
    echo $((diff / 86400))
    unset epoch1 epoch2 diff
}

# Add days to an ISO 8601 date (YYYY-MM-DD) using leap-year-aware arithmetic.
# Usage: add_days <date> <num_days>
# Returns the new date as YYYY-MM-DD
add_days() {
    date_str="$1"
    days_to_add="$2"

    # Parse input date using sed to strip leading zeros
    year=$(echo "$date_str" | sed 's/^\([0-9]*\)-.*/\1/' | sed 's/^0*//')
    month=$(echo "$date_str" | sed 's/^[0-9]*-\([0-9]*\)-.*/\1/' | sed 's/^0*//')
    day=$(echo "$date_str" | sed 's/^[0-9]*-[0-9]*-\([0-9]*\)$/\1/' | sed 's/^0*//')

    # Handle empty values (leading zero stripped to nothing)
    [ -z "$year" ] && year=0
    [ -z "$month" ] && month=0
    [ -z "$day" ] && day=0

    # Validate input
    if ! { [ "$month" -ge 1 ] && [ "$month" -le 12 ]; } || ! { [ "$day" -ge 1 ] && [ "$day" -le 31 ]; }; then
        unset year month day max_days
        return 1
    fi

    # Add days directly to day field
    day=$((day + days_to_add))

    # Handle month/year overflow with leap-year awareness
    while [ "$day" -gt 0 ]; do
        max_days=$(days_in_month "$month" "$year")

        if [ "$day" -le "$max_days" ]; then
            break
        fi

        # Move to next month
        day=$((day - max_days))
        month=$((month + 1))

        if [ "$month" -gt 12 ]; then
            month=1
            year=$((year + 1))
        fi
    done

    # Handle underflow (negative days from subtraction)
    while [ "$day" -le 0 ]; do
        month=$((month - 1))

        if [ "$month" -lt 1 ]; then
            month=12
            year=$((year - 1))
        fi

        max_days=$(days_in_month "$month" "$year")
        day=$((day + max_days))
    done

    # Format as YYYY-MM-DD
    printf '%04d-%02d-%02d\n' "$year" "$month" "$day"
    unset year month day max_days
}
