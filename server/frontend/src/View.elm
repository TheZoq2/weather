module View exposing (view)

-- Standard imports
import Model exposing (Model)
import Msg exposing (Msg(..))

-- External imports
import Dict exposing (Dict)
import Html
import Html.Styled exposing (..)
import Html.Styled.Events exposing (..)
import Html.Styled.Attributes exposing (..)
import Svg.Styled as Svg exposing (svg)
import Svg.Styled.Attributes as SvgAttributes exposing (viewBox)
import Graph
import Time exposing (Time, second)
import List.Extra

-- Project imports
import Constants exposing (day)
import Style exposing
    ( styledButton
    , valueContainer
    , valueContainerItem
    , contentContainer
    , singleValue
    , singleValueContainer
    )

view : Model -> Html.Html Msg
view model =
    toUnstyled <| contentContainer
        []
        (  [ dataSelector model.availableData
           , timeSelectionButtons
           , valueContainer
                []
                <| [singleValueDisplay model.values]
                   ++ (drawValues model.timeRange model.values)
           ]
        )


singleValueDisplay : Dict String (List (Time, Float)) -> Html Msg
singleValueDisplay values =
    valueContainerItem "single_values"
        []
        <| [ singleValueContainer
               []
               <| List.map
                   (\(name, values) ->
                       let
                           {symbol, unitName, rounding} = readingProperties name

                           latestValue = rounding
                               <| Tuple.second
                               <| Maybe.withDefault (0, 0)
                               <| List.head
                               <| List.reverse values
                       in
                           singleValue symbol latestValue unitName
                   )
               <| Dict.toList values
           ]

drawValues : Time -> Dict String (List (Time, Float)) -> List (Html Msg)
drawValues timeRange values =
    let
        graphParamFn : ReadingProperty -> List (Time, Float) -> GraphParams
        graphParamFn readingProperty values =
            GraphParams 600 readingProperty.graphHeight (readingProperty.valueRangeFn values) readingProperty.separation readingProperty.unitName

    in
        List.map
            (\(name, values) ->
                let
                    maxTime = List.map Tuple.first values |> List.maximum |> Maybe.withDefault 0
                    minTime = maxTime - timeRange
                    startEndTime = (minTime, maxTime)

                    filter (time, val) = 
                        (time <= maxTime && time >= minTime)

                    displayedValues = List.filter filter values

                    readingProperty = readingProperties name
                    processedValues = readingProperty.preprocessor displayedValues

                    graphParams = graphParamFn readingProperty processedValues
                in
                    valueContainerItem name
                        []
                        ( [ p [] [text name]
                          ]
                          ++ (drawGraph graphParams startEndTime processedValues)
                        )
            )
            <| Dict.toList values


type alias GraphParams =
    { viewWidth: Int
    , viewHeight: Int
    , valueRange: (Float, Float)
    , horizontalStep: Float
    , unit: String
    }

drawGraph : GraphParams -> (Time, Time) -> List (Time, Float) ->  List (Html Msg)
drawGraph {viewWidth, viewHeight, valueRange, horizontalStep, unit} startEndTime values =
    let
        viewDimensions = (viewWidth, viewHeight)
    in
        [ svg
          [ viewBox <| "0 0 " ++ "20" ++ " " ++ (toString viewHeight)
          , SvgAttributes.width <| toString 40 ++ "px"
          , SvgAttributes.height <| toString viewHeight ++ "px"
          ]
          [ Svg.fromUnstyled <| Graph.drawLegend unit viewHeight valueRange horizontalStep
          ]
        , svg
          [ viewBox <| "0 0 " ++ (toString viewWidth) ++ " " ++ (toString viewHeight)
          , SvgAttributes.width <| toString viewWidth ++ "px"
          , SvgAttributes.height <| toString viewHeight ++ "px"
          ]
          [ Svg.fromUnstyled <| Graph.drawHorizontalLines viewDimensions valueRange horizontalStep
          , Svg.fromUnstyled <| Graph.drawGraph viewDimensions valueRange startEndTime values
          ]
        ]



dataSelector : List String -> Html Msg
dataSelector availableData =
    let
        links = List.map (\name -> styledButton [onClick (ToggleData name)] [text name]) availableData
    in
        div [] links


timeSelectionButtons : Html Msg
timeSelectionButtons =
    div []
        [ styledButton [onClick <| TimeRangeChanged (7*day)] [text "7 days"]
        , styledButton [onClick <| TimeRangeChanged day] [text "24 hours"]
        , styledButton [onClick <| TimeRangeChanged (12 * Time.hour)] [text "12 hours"]
        , styledButton [onClick <| TimeRangeChanged (6 * Time.hour)] [text "6 hours"]
        , styledButton [onClick <| TimeRangeChanged (1 * Time.hour)] [text "1 hour"]
        ]


type alias ReadingProperty =
    { valueRangeFn: List (Time, Float) -> (Float, Float)
    , preprocessor: List (Time, Float) -> List(Time, Float)
    , separation: Float
    , unitName: String
    , graphHeight: Int
    , symbol: String
    , rounding: Float -> Float
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
        lastNValues n list =
            List.drop ((List.length list) - n) list

        minMaxWithLimits : Float -> List (Time, Float) -> (Float, Float) 
        minMaxWithLimits minRange values =
            let
                min = Maybe.withDefault 0 <| List.minimum <| List.map Tuple.second <| values
                max = Maybe.withDefault 0 <| List.maximum <| List.map Tuple.second <| values

                range = max-min
                padding = Maybe.withDefault 0 <| List.maximum [0, (minRange - range) / 2]
            in
                (min-padding, max+padding)

        roundToFloat : Float -> Float -> Float
        roundToFloat decimals value =
            (toFloat <| round <| value * (10 ^ decimals)) / (10 ^ decimals)

        roundToInteger value =
            toFloat <| round <| value

        independent minRange unitName separation symbol rounding =
            { valueRangeFn = minMaxWithLimits minRange
            , preprocessor = identity
            , separation = separation
            , unitName = unitName
            , graphHeight = 250
            , symbol = symbol
            , rounding = roundToFloat 0
            }

    in
        case name of
            "humidity" -> independent 10 "%" 10 "💧" roundToInteger
            "temperature" -> independent 10 "°C" 5 "🌡" (roundToFloat 1)
            "wind_raw" -> independent 0.5 "wU" 0.5 "🍃" (roundToFloat 2)
            "battery" -> independent 4.2 "V" 0.5 "🔋" (roundToFloat 2)
            _ ->
                { valueRangeFn = (\_ -> (0, 100))
                , preprocessor = (\list -> list)
                , separation = 5
                , unitName = "-"
                , graphHeight = 300
                , symbol = "⯑"
                , rounding = roundToInteger
                }


