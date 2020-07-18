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
import List.Extra
import String exposing (fromInt)
import Browser exposing (Document)

-- Project imports
import Constants exposing (week, day, hour)
import Style exposing
    ( styledButton
    , valueContainer
    , valueContainerItem
    , contentContainer
    , singleValue
    , singleValueContainer
    )

view : Model -> Document Msg
view model =
    let
        controls = valueContainerItem "controls" []
            [ dataSelector model.availableData
            , timeSelectionButtons
            ]
    in
        { title = "weather"
        , body = [
            toUnstyled <| contentContainer
                []
                 [ valueContainer
                      []
                      <| [controls, singleValueDisplay model.values]
                         ++ drawValues model.timeRange model.values
                 ]
             ]
        }


singleValueDisplay : Dict String (List (Int, Float)) -> Html Msg
singleValueDisplay values =
    valueContainerItem "single_values"
        []
        <| [ singleValueContainer
               []
               <| List.map
                   (\(name, values_) ->
                       let
                           {symbol, unitName, rounding} = readingProperties name

                           latestValue = rounding
                               <| Tuple.second
                               <| Maybe.withDefault (0, 0)
                               <| List.head
                               <| List.reverse values_
                       in
                           singleValue symbol latestValue unitName
                   )
               <| Dict.toList values
           ]

drawValues : Int -> Dict String (List (Int, Float)) -> List (Html Msg)
drawValues timeRange values =
    let
        graphParamFn : ReadingProperty -> List (Int, Float) -> GraphParams
        graphParamFn readingProperty values_ =
            GraphParams 600 readingProperty.graphHeight (readingProperty.valueRangeFn values_) readingProperty.separation readingProperty.unitName

    in
        List.map
            (\(name, values_) ->
                let
                    maxTime = List.map Tuple.first values_ |> List.maximum |> Maybe.withDefault 0
                    minTime = maxTime - timeRange
                    startEndTime = (minTime, maxTime)

                    filter (time, val) = 
                        time <= maxTime && time >= minTime

                    displayedValues = List.filter filter values_

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

drawGraph : GraphParams -> (Int, Int) -> List (Int, Float) ->  List (Html Msg)
drawGraph {viewWidth, viewHeight, valueRange, horizontalStep, unit} startEndTime values =
    let
        viewDimensions = (viewWidth, viewHeight)

        startEndTimeInt = Tuple.mapBoth toFloat toFloat startEndTime
    in
        [ svg
            [ viewBox <| "0 0 " ++ "20" ++ " " ++ (fromInt viewHeight)
            , SvgAttributes.width <| fromInt 40 ++ "px"
            , SvgAttributes.height <| fromInt viewHeight ++ "px"
            ]
            [ Svg.fromUnstyled <| Graph.drawLegend unit viewHeight valueRange horizontalStep
            ]
        , svg
            [ viewBox <| "0 0 " ++ (fromInt viewWidth) ++ " " ++ (fromInt viewHeight)
            , SvgAttributes.width <| fromInt viewWidth ++ "px"
            , SvgAttributes.height <| fromInt viewHeight ++ "px"
            ]
            [ Svg.fromUnstyled <| Graph.drawHorizontalLines viewDimensions valueRange horizontalStep
            , Svg.fromUnstyled
                <| Graph.drawGraph viewDimensions valueRange startEndTimeInt
                <| List.map (Tuple.mapFirst toFloat) values 
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
        [ styledButton [onClick <| TimeRangeChanged (week)] [text "7 days"]
        , styledButton [onClick <| TimeRangeChanged day] [text "24 hours"]
        , styledButton [onClick <| TimeRangeChanged (12 * hour)] [text "12 hours"]
        , styledButton [onClick <| TimeRangeChanged (6 * hour)] [text "6 hours"]
        , styledButton [onClick <| TimeRangeChanged (1 * hour)] [text "1 hour"]
        ]


type alias ReadingProperty =
    { valueRangeFn: List (Int, Float) -> (Float, Float)
    , preprocessor: List (Int, Float) -> List(Int, Float)
    , separation: Float
    , unitName: String
    , graphHeight: Int
    , symbol: String
    , rounding: Float -> Float
    }



readingProperties : String -> ReadingProperty
readingProperties name =
    let
        minMaxWithLimits : Float -> List (Int, Float) -> (Float, Float) 
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
            , rounding = rounding
            }
    in
        case name of
            "humidity" -> independent 10 "%" 10 "💧" roundToInteger
            "temperature" -> independent 10 "°C" 5 "🌡" (roundToFloat 1)
            "wind_raw" -> independent 0.5 "wU" 0.5 "🍃" (roundToFloat 2)
            "battery" -> independent 1.0 "V" 0.5 "🔋" (roundToFloat 1)
            _ ->
                { valueRangeFn = (\_ -> (0, 100))
                , preprocessor = (\list -> list)
                , separation = 5
                , unitName = "-"
                , graphHeight = 300
                , symbol = "⯑"
                , rounding = roundToInteger
                }


