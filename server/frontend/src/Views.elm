module Views exposing (timeSelectionButtons)

import Html exposing (..)
import Html.Events exposing (..)
import Time exposing (Time)

import Msg exposing (Msg(..))
import Constants exposing (week, day)

timeSelectionButtons : Html Msg
timeSelectionButtons =
    div []
        [ button [onClick <| TimeRangeChanged (7*day)] [text "7 days"]
        , button [onClick <| TimeRangeChanged day] [text "24 hours"]
        , button [onClick <| TimeRangeChanged (12 * Time.hour)] [text "12 hours"]
        , button [onClick <| TimeRangeChanged (6 * Time.hour)] [text "6 hours"]
        , button [onClick <| TimeRangeChanged (1 * Time.hour)] [text "1 hour"]
        ]


