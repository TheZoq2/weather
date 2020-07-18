module Model exposing (Model)

import Time exposing (Posix)
import Dict exposing (Dict)

type alias Model =
    { values: Dict String (List (Int, Float))
    , listedData: List String
    , availableData: List String
    -- Url of the server
    , url: String
    , timeRange: Int
    }
