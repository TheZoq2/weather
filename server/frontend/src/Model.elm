module Model exposing (Model)

import Time exposing (Time)
import Dict exposing (Dict)

type alias Model =
    { values: Dict String (List (Time, Float))
    , listedData: List String
    , availableData: List String
    -- Url of the server
    , url: String
    , timeRange: Time
    }
