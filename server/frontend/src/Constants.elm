module Constants exposing (day, week, hour)

-- A day in milliseconds
hour : Int
hour = 60*60*1000

day : Int
day = hour * 24

week : Int
week = day * 7
