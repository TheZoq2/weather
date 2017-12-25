module Main exposing (..)

import Html exposing (..)
import Html.Attributes
import Http
import Time exposing (Time, second)
import Json.Decode as Decode
import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)

type alias Model =
    { temperature: List Float
    }


init : (Model, Cmd Msg)
init =
    (Model [], Cmd.none)


type Msg
    = TemperaturesReceived (Result Http.Error (List Float))
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


decodeTemperatures : Decode.Decoder (List Float)
decodeTemperatures =
    Decode.list Decode.float

getTemperatures : Http.Request (List Float)
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
    in
    div
        []
        ( [ svg
            [ viewBox <| "0 -30" ++ (toString viewWidth) ++ "70"
            , width <| toString viewWidth ++ "px"
            , height <| toString viewHeight ++ "px"
            ]
            [ drawGraph viewDimensions (0, 40) model.temperature
            ]
        ]
        ++ case List.head <| List.reverse model.temperature of
                Just value ->
                    [h1 [] [Html.text <| toString value]]
                Nothing ->
                    []
        )





drawGraph : (Int, Int) -> (Float, Float) -> List Float -> Svg Msg
drawGraph (viewW, viewH) (min, max) data =
    let
        x_points =
            List.range 0 (List.length data)
            |> List.map (\x -> toFloat x / toFloat (List.length data) * (toFloat viewW))

        pointsString =
            List.map (\y -> (toFloat viewH) * (y + min) / max - (min/max)) data
            |> List.map (\y -> (toFloat viewH) - y)
            |> List.map2 (,) x_points
            |> List.map (\(x,y) -> toString x ++ "," ++ toString y)
            |> List.intersperse " "
            |> String.concat

    in
        polyline [fill "none", stroke "black", points pointsString] []

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


