module Requests exposing (..)

import Json.Decode as Decode
import Time exposing (Time, second)
import Http
import Msg exposing (Msg(..))

decodeTemperatures : Decode.Decoder (List (Time, Float))
decodeTemperatures =
    let
        timestampDecoder = Decode.field "timestamp" (Decode.map ((*) Time.second) Decode.float)
        valueDecoder = Decode.field "value" Decode.float
    in
        Decode.list <| Decode.map2 (,) timestampDecoder valueDecoder

getValues : String -> String -> Http.Request (List (Time, Float))
getValues url name =
    Http.get ("http://" ++ url ++ "/data/" ++ name) decodeTemperatures

sendValueRequest : String -> String -> Cmd Msg
sendValueRequest url name =
    Http.send (ValuesReceived name) (getValues url name)

getAvailableData : String -> Http.Request (List String)
getAvailableData url =
    Http.get ("http://" ++ url ++ "/data") <| Decode.list Decode.string

sendAvailableDataQuery : String -> Cmd Msg
sendAvailableDataQuery url =
    Http.send AvailableDataReceived <| getAvailableData url
