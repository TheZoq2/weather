module Main exposing (..)

import Html exposing (..)
import Html.Attributes
import Html.Events
import Http
import Time exposing (Time, second)
import Json.Decode as Decode
import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)
import Dict exposing (Dict)
import List.Extra

import Graph
import Time

type alias ReadingProperty =
    { valueRangeFn: List (Time, Float) -> (Float, Float)
    , preprocessor: List (Time, Float) -> List(Time, Float)
    , separation: Float
    , unitName: String
    , graphHeight: Int
    }


stepPreprocessor : List (Time, Float) -> List (Time, Float)
stepPreprocessor original =
    let
        duplicated = List.Extra.interweave original original

        (times, values) = List.unzip duplicated

        shiftedTimes = List.drop 1 times
    in
        List.Extra.zip shiftedTimes values

readingProperties : String -> ReadingProperty
readingProperties name =
    let
        binaryReading =
            { valueRangeFn = (\_ -> (0, 1))
            , preprocessor = stepPreprocessor
            , separation = 1
            , unitName = ""
            , graphHeight = 50
            }
    in
        case name of
            "channel1" -> binaryReading
            "channel2" -> binaryReading
            _ ->
                { valueRangeFn = (\_ -> (-15, 40))
                , preprocessor = (\list -> list)
                , separation = 5
                , unitName = "°C"
                , graphHeight = 300
                }


type alias Model =
    { values: Dict String (List (Time, Float))
    , listedData: List String
    , availableData: List String
    }


init : (Model, Cmd Msg)
init =
    (Model Dict.empty [] [], Cmd.none)


type Msg
    = ValuesReceived String (Result Http.Error (List (Time, Float)))
    | Tick Time
    | AvailableDataReceived (Result Http.Error (List String))
    | ToggleData String


update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        ValuesReceived name values ->
            case values of
                Ok values ->
                    let
                        newValues = Dict.insert name values model.values
                    in
                        ({model | values = newValues}, Cmd.none)
                Err e ->
                    let
                        _ = Debug.log "Failed to make http request" e
                    in
                        (model, Cmd.none)
        Tick time ->
            let
                requests = sendAvailableDataQuery :: (List.map sendValueRequest model.listedData)
            in
                (model, Cmd.batch requests)
        AvailableDataReceived data ->
            case data of
                Ok availableData ->
                    ({model | availableData = availableData}, Cmd.none)
                Err e ->
                    let
                        _ = Debug.log "Failed to get available data" e
                    in
                        (model, Cmd.none)
        ToggleData name ->
            let
                newListed = 
                    if List.member name model.listedData then
                        List.filter (\x -> x == name) model.listedData
                    else
                        name :: model.listedData
            in
                ({model | listedData = newListed}, Cmd.none)


decodeTemperatures : Decode.Decoder (List (Time, Float))
decodeTemperatures =
    let
        timestampDecoder = Decode.field "timestamp" (Decode.map ((*) Time.second) Decode.float)
        valueDecoder = Decode.field "value" Decode.float
    in
        Decode.list <| Decode.map2 (,) timestampDecoder valueDecoder

getValues : String -> Http.Request (List (Time, Float))
getValues name =
    Http.get ("http://localhost:8080/data/" ++ name) decodeTemperatures

sendValueRequest : String -> Cmd Msg
sendValueRequest name =
    Http.send (ValuesReceived name) (getValues name)

getAvailableData : Http.Request (List String)
getAvailableData =
    Http.get "http://localhost:8080/data" <| Decode.list Decode.string

sendAvailableDataQuery : Cmd Msg
sendAvailableDataQuery =
    Http.send AvailableDataReceived getAvailableData

view : Model -> Html Msg
view model =
    div
        []
        (  [dataSelector model.availableData]
        ++ drawValues model.values
        )


drawValues : Dict String (List (Time, Float)) -> List (Html Msg)
drawValues values =
    let
        graphParamFn : ReadingProperty -> List (Time, Float) -> GraphParams
        graphParamFn readingProperty values =
            GraphParams 600 readingProperty.graphHeight (readingProperty.valueRangeFn values) readingProperty.separation
    in
        List.map
            (\(name, values) ->
                let
                    readingProperty = readingProperties name
                    processedValues = readingProperty.preprocessor values

                    graphParams = graphParamFn readingProperty values
                in
                    div
                        []
                        ( [ p [] [Html.text name]
                          ]
                          ++ (drawGraph graphParams processedValues)
                        )
            )
            <| Dict.toList values


type alias GraphParams =
    { viewWidth: Int
    , viewHeight: Int
    , valueRange: (Float, Float)
    , horizontalStep: Float
    }

drawGraph : GraphParams ->  List (Time, Float) -> List (Html Msg)
drawGraph {viewWidth, viewHeight, valueRange, horizontalStep} values =
    let
        viewDimensions = (viewWidth, viewHeight)
    in
        [ svg
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
          , Graph.drawGraph viewDimensions valueRange values
          ]
        ]



dataSelector : List String -> Html Msg
dataSelector availableData =
    let
        links = List.map (\name -> Html.button [Html.Events.onClick (ToggleData name)] [Html.text name]) availableData
    in
        li [] links



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


