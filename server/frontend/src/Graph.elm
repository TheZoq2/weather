module Graph exposing (drawGraph)

import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)



transformToGraphCoordinates : Float -> (Float, Float) -> Float -> Float
transformToGraphCoordinates viewHeight (minVal, maxVal) val =
    let
        minMaxRange = maxVal - minVal
    in
        viewHeight * ((val - minVal) / minMaxRange)



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

