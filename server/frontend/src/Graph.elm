module Graph exposing (drawGraph, drawHorizontalLines, drawLegend)

import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)



transformToGraphCoordinates : Float -> (Float, Float) -> Float -> Float
transformToGraphCoordinates viewHeight (minVal, maxVal) val =
    let
        minMaxRange = maxVal - minVal
    in
        viewHeight - (viewHeight * ((val - minVal) / minMaxRange))



drawGraph : (Int, Int) -> (Float, Float) -> List Float -> Svg a
drawGraph (viewW, viewH) (min, max) data =
    let
        x_points =
            List.range 0 (List.length data)
            |> List.map (\x -> toFloat x / toFloat (List.length data) * (toFloat viewW))

        pointsString =
            List.map (transformToGraphCoordinates (toFloat viewH) (min, max)) data
            |> List.map2 (,) x_points
            |> List.map (\(x,y) -> toString x ++ "," ++ toString y)
            |> List.intersperse " "
            |> String.concat
    in
        polyline [fill "none", stroke "black", points pointsString] []


drawHorizontalLines : (Int, Int) -> (Float, Float) -> Float -> Svg a
drawHorizontalLines (viewW, viewH) valueRange verticalStep =
    let
        yCoords = (getHorizontalFixpoints viewH valueRange verticalStep)
    in
        List.map toString yCoords
        |> List.map (\y -> line [x1 "0", x2 <| toString viewW, y1 y, y2 y] [])
        |> g [stroke "lightgray"]

drawLegend : String -> Int -> (Float, Float) -> Float -> Svg a
drawLegend unit viewH (min, max) verticalStep =
    let
        yCoords = (getHorizontalFixpoints viewH (min, max) verticalStep)

        yValues =
            List.range 0 (List.length yCoords)
            |> List.map (\y -> (toFloat y) * verticalStep + min)
    in
        List.map2 (,) yCoords yValues
        |> List.map (\(yCoord, yVal) ->
                text_ [y <| toString yCoord, fontSize "10px"] [text <| toString yVal ++ unit] ) 
        |> g []


getHorizontalFixpoints : Int -> (Float, Float) -> Float -> List Float
getHorizontalFixpoints viewH (min, max) verticalStep =
    let
        stepStart = (floor (min/verticalStep))
        stepEnd = (ceiling (max/verticalStep))
    in
        List.range stepStart stepEnd
        |> List.map toFloat
        |> List.map ((*) verticalStep)
        |> List.map (transformToGraphCoordinates (toFloat viewH) (min,max))


