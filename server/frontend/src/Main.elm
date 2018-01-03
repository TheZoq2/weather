module Main exposing (..)

import Html exposing (..)
import Html.Attributes
import Http
import Time exposing (Time, second)
import Json.Decode as Decode
import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)

import Graph
import Time

type alias Model =
    { temperature: List (Time, Float)
    }


init : (Model, Cmd Msg)
init =
    (Model [], Cmd.none)


type Msg
    = TemperaturesReceived (Result Http.Error (List (Time, Float)))
    | Tick Time


update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        TemperaturesReceived temperature ->
            case temperature of
                Ok temperature ->
                    ({model | temperature = temperature}, Cmd.none)
                Err e ->
                    let
                        _ = Debug.log "Failed to make http request" e
                    in
                        (model, Cmd.none)
        Tick time ->
            (model, sendTemperatureRequest)


decodeTemperatures : Decode.Decoder (List (Time, Float))
decodeTemperatures =
    let
        timestampDecoder = Decode.field "timestamp" (Decode.map ((*) Time.second) Decode.float)
        valueDecoder = Decode.field "value" Decode.float
    in
        Decode.list <| Decode.map2 (,) timestampDecoder valueDecoder

getTemperatures : Http.Request (List (Time, Float))
getTemperatures =
    Http.get "http://localhost:8080/data/temperature" decodeTemperatures

sendTemperatureRequest : Cmd Msg
sendTemperatureRequest =
    Http.send TemperaturesReceived getTemperatures

view : Model -> Html Msg
view model =
    let
        viewWidth = 600
        viewHeight = 400

        viewDimensions = (viewWidth, viewHeight)

        valueRange = (-10, 40)

        horizontalStep = 5
    in
    div
        []
        ( [ svg
            [ viewBox <| "0 0 " ++ "20" ++ " " ++ (toString viewHeight)
            , width <| toString 40 ++ "px"
            , height <| toString viewHeight ++ "px"
            ]
            [ Graph.drawLegend "°C" viewHeight valueRange horizontalStep
            ]
          , svg
            [ viewBox <| "0 0 " ++ (toString viewWidth) ++ " " ++ (toString viewHeight)
            , width <| toString viewWidth ++ "px"
            , height <| toString viewHeight ++ "px"
            ]
            [ Graph.drawHorizontalLines viewDimensions valueRange horizontalStep
            , Graph.drawGraph viewDimensions valueRange (List.map Tuple.second model.temperature)
            ]
          ]
        ++ case List.head <| List.reverse model.temperature of
                Just value ->
                    [h1 [] [Html.text <| toString value]]
                Nothing ->
                    []
        )





subscriptions : Model -> Sub Msg
subscriptions model =
    Time.every second Tick


main : Program Never Model Msg
main =
    Html.program
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


