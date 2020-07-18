module Msg exposing (Msg(..))

import Http
import Time exposing (Posix)
import Url exposing (Url)

type Msg
    = ValuesReceived String (Result Http.Error (List (Int, Float)))
    | Tick Posix
    | AvailableDataReceived (Result Http.Error (List String))
    | ToggleData String
    | UrlChanged Url
    | TimeRangeChanged Int
    | Dummy

