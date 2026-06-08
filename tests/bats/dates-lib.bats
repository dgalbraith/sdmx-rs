#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/lib/dates.sh
#
# Unit tests for portable date utilities.
# Validates leap year handling, date arithmetic, and epoch conversions.
#
# Run with: bats tests/bats/dates-lib.bats
# ==============================================================================

setup() {
    # Source the dates library
    source "scripts/lib/dates.sh"
}

# ==============================================================================
# Leap Year Detection
# ==============================================================================

@test "is_leap_year: 2024 is a leap year" {
    is_leap_year 2024
    [ "$?" -eq 0 ]
}

@test "is_leap_year: 2025 is not a leap year" {
    ! is_leap_year 2025
}

@test "is_leap_year: 2026 is not a leap year" {
    ! is_leap_year 2026
}

@test "is_leap_year: 2000 is a leap year (divisible by 400)" {
    is_leap_year 2000
    [ "$?" -eq 0 ]
}

@test "is_leap_year: 1900 is not a leap year (divisible by 100 but not 400)" {
    ! is_leap_year 1900
}

@test "is_leap_year: 2028 is a leap year" {
    is_leap_year 2028
    [ "$?" -eq 0 ]
}

# ==============================================================================
# Days in Month (with Leap Year Handling)
# ==============================================================================

@test "days_in_month: January has 31 days" {
    result=$(days_in_month 1 2026)
    [ "$result" -eq 31 ]
}

@test "days_in_month: February has 28 days in non-leap year" {
    result=$(days_in_month 2 2026)
    [ "$result" -eq 28 ]
}

@test "days_in_month: February has 29 days in leap year" {
    result=$(days_in_month 2 2024)
    [ "$result" -eq 29 ]
}

@test "days_in_month: April has 30 days" {
    result=$(days_in_month 4 2026)
    [ "$result" -eq 30 ]
}

@test "days_in_month: December has 31 days" {
    result=$(days_in_month 12 2026)
    [ "$result" -eq 31 ]
}

# ==============================================================================
# Date Arithmetic: add_days
# ==============================================================================

@test "add_days: 2026-08-15 + 30 days = 2026-09-14" {
    result=$(add_days "2026-08-15" 30)
    [ "$result" = "2026-09-14" ]
}

@test "add_days: 2026-08-15 + 1 day = 2026-08-16" {
    result=$(add_days "2026-08-15" 1)
    [ "$result" = "2026-08-16" ]
}

@test "add_days: 2026-08-31 + 1 day = 2026-09-01" {
    result=$(add_days "2026-08-31" 1)
    [ "$result" = "2026-09-01" ]
}

@test "add_days: 2026-12-31 + 1 day = 2027-01-01" {
    result=$(add_days "2026-12-31" 1)
    [ "$result" = "2027-01-01" ]
}

@test "add_days: 2026-01-31 + 1 day = 2026-02-01" {
    result=$(add_days "2026-01-31" 1)
    [ "$result" = "2026-02-01" ]
}

# ==============================================================================
# Date Arithmetic: Leap Year Boundaries
# ==============================================================================

@test "add_days: leap year Feb boundary - 2024-02-28 + 1 day = 2024-02-29" {
    result=$(add_days "2024-02-28" 1)
    [ "$result" = "2024-02-29" ]
}

@test "add_days: leap year Feb boundary - 2024-02-29 + 1 day = 2024-03-01" {
    result=$(add_days "2024-02-29" 1)
    [ "$result" = "2024-03-01" ]
}

@test "add_days: non-leap year Feb boundary - 2026-02-28 + 1 day = 2026-03-01" {
    result=$(add_days "2026-02-28" 1)
    [ "$result" = "2026-03-01" ]
}

@test "add_days: century leap year (2000) - 2000-02-28 + 1 day = 2000-02-29" {
    result=$(add_days "2000-02-28" 1)
    [ "$result" = "2000-02-29" ]
}

@test "add_days: century non-leap year (1900) - 1900-02-28 + 1 day = 1900-03-01" {
    result=$(add_days "1900-02-28" 1)
    [ "$result" = "1900-03-01" ]
}

# ==============================================================================
# Date Arithmetic: Multiple Month Wrapping
# ==============================================================================

@test "add_days: 2026-01-15 + 45 days = 2026-03-01" {
    result=$(add_days "2026-01-15" 45)
    [ "$result" = "2026-03-01" ]
}

@test "add_days: 2026-11-15 + 50 days = 2027-01-04" {
    result=$(add_days "2026-11-15" 50)
    [ "$result" = "2027-01-04" ]
}

@test "add_days: 2024-01-15 + 365 days = 2025-01-14 (leap year has 366 days)" {
    result=$(add_days "2024-01-15" 365)
    [ "$result" = "2025-01-14" ]
}

# ==============================================================================
# Date Arithmetic: Zero and Negative (Underflow Handling)
# ==============================================================================

@test "add_days: adding zero days returns same date" {
    result=$(add_days "2026-08-15" 0)
    [ "$result" = "2026-08-15" ]
}

@test "add_days: negative days (going backward) - 2026-03-01 - 1 day = 2026-02-28" {
    result=$(add_days "2026-03-01" -1)
    [ "$result" = "2026-02-28" ]
}

@test "add_days: negative days across year boundary - 2026-01-01 - 1 day = 2025-12-31" {
    result=$(add_days "2026-01-01" -1)
    [ "$result" = "2025-12-31" ]
}

@test "add_days: negative days leap year - 2024-03-01 - 1 day = 2024-02-29" {
    result=$(add_days "2024-03-01" -1)
    [ "$result" = "2024-02-29" ]
}

# ==============================================================================
# Date Arithmetic: Edge Cases
# ==============================================================================

@test "add_days: large positive offset - 2000-01-01 + 10000 days ≈ 2027-05-19" {
    result=$(add_days "2000-01-01" 10000)
    [ "$result" = "2027-05-19" ]
}

@test "add_days: first day of year + 1 = second day of year" {
    result=$(add_days "2026-01-01" 1)
    [ "$result" = "2026-01-02" ]
}

@test "add_days: last day of year + 1 = first day of next year" {
    result=$(add_days "2026-12-31" 1)
    [ "$result" = "2027-01-01" ]
}

# ==============================================================================
# Days Between Dates
# ==============================================================================

@test "days_between: same date = 0 days" {
    result=$(days_between "2026-08-15" "2026-08-15")
    [ "$result" -eq 0 ]
}

@test "days_between: 2026-08-15 to 2026-09-14 = 30 days" {
    result=$(days_between "2026-08-15" "2026-09-14")
    [ "$result" -eq 30 ]
}

@test "days_between: 2026-08-15 to 2026-08-16 = 1 day" {
    result=$(days_between "2026-08-15" "2026-08-16")
    [ "$result" -eq 1 ]
}

@test "days_between: reversed dates = negative result" {
    result=$(days_between "2026-09-14" "2026-08-15")
    [ "$result" -eq -30 ]
}

@test "days_between: crossing leap year Feb - 2024-02-28 to 2024-03-01 = 2 days" {
    result=$(days_between "2024-02-28" "2024-03-01")
    [ "$result" -eq 2 ]
}

@test "days_between: year boundary - 2026-12-31 to 2027-01-01 = 1 day" {
    result=$(days_between "2026-12-31" "2027-01-01")
    [ "$result" -eq 1 ]
}

# ==============================================================================
# Date to Epoch Conversion
# ==============================================================================

@test "date_to_epoch: 1970-01-01 to 1970-01-02 = 86400 seconds (1 day difference)" {
    epoch1=$(date_to_epoch "1970-01-01")
    epoch2=$(date_to_epoch "1970-01-02")
    diff=$((epoch2 - epoch1))
    [ "$diff" -eq 86400 ]
}

@test "date_to_epoch: 2026-08-15 to 2026-08-16 = 86400 seconds (1 day difference)" {
    epoch1=$(date_to_epoch "2026-08-15")
    epoch2=$(date_to_epoch "2026-08-16")
    diff=$((epoch2 - epoch1))
    [ "$diff" -eq 86400 ]
}

@test "date_to_epoch: 2026-08-15 is positive integer" {
    result=$(date_to_epoch "2026-08-15")
    [ "$result" -gt 0 ]
}

@test "date_to_epoch: later dates have larger epoch values" {
    epoch1=$(date_to_epoch "2026-08-15")
    epoch2=$(date_to_epoch "2026-09-14")
    [ "$epoch2" -gt "$epoch1" ]
}

# ==============================================================================
# Input Validation
# ==============================================================================

@test "add_days: invalid month (13) fails" {
    ! add_days "2026-13-15" 1
}

@test "add_days: invalid day (0) fails" {
    ! add_days "2026-08-00" 1
}

@test "date_to_epoch: invalid month fails" {
    ! date_to_epoch "2026-13-15"
}

@test "date_to_epoch: invalid day fails" {
    ! date_to_epoch "2026-08-32"
}

@test "days_between: invalid dates fail" {
    ! days_between "2026-13-15" "2026-08-15"
}

@test "date_to_epoch: multiple leading zeros on year, month, and day are handled" {
    run date_to_epoch "0099-008-0005"
    [ "$status" -ne 0 ]
    epoch1=$(date_to_epoch "2026-08-05")
    epoch2=$(date_to_epoch "2026-008-0005")
    [ "$epoch1" -eq "$epoch2" ]
}

@test "add_days: multiple leading zeros on year, month, and day are stripped correctly" {
    result=$(add_days "2026-008-0005" 1)
    [ "$result" = "2026-08-06" ]
}

@test "date_to_epoch: dates prior to 1970 fail explicitly" {
    run date_to_epoch "1969-12-31"
    [ "$status" -ne 0 ]
}
