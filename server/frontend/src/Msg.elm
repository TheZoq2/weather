module Msg exposing (Msg(..))

import Http
import Time exposing (Time)
import Navigation

type Msg
    = ValuesReceived String (Result Http.Error (List (Time, Float)))
    | Tick Time
    | AvailableDataReceived (Result Http.Error (List String))
    | ToggleData String
    | UrlChanged Navigation.Location
    | TimeRangeChanged Time

