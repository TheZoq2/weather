module Constants exposing (day, week)

import Time exposing (Time)

day : Time
day =
    Time.hour * 24

week : Time
week =
    day * 7
