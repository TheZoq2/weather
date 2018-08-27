module Style exposing (..)

-- Standard library imports
import Dict exposing (Dict)

-- External imports
import Css exposing (..)
import Html.Styled exposing (..)

-- Project imports




-- Common styles

{-|
  Applied to the outer div of the body
-}
contentContainer : List (Attribute msg) -> List (Html msg) -> Html msg
contentContainer =
    styled div
        [ fontFamilies ["sans-serif"]
        , backgroundColor (hex "f2f8ff")
        ]


{-|
  Applied to the container of the 'cards' on the page
-}
valueContainer : List (Attribute msg) -> List (Html msg) -> Html msg
valueContainer =
    styled div
        [ margin2 (px 10) (px 0)
        , displayFlex
        , flexWrap wrap
        , justifyContent center
        ]


{-|
  Applied to each 'card' in the page
-}
valueContainerItem : String -> List (Attribute msg) -> List (Html msg) -> Html msg
valueContainerItem name =
    let
        highlightColor = Maybe.withDefault (hex "#36de1c")
            <| Dict.get name
                <| Dict.fromList
                    [ ("temperature", hex "de521c")
                    , ("humidity", hex "460080")
                    , ("wind_raw", hex "00c610")
                    , ("single_values", hex "f4ff40")
                    ]
    in
        styled div
            [ padding (px 5)
            , margin (px 5)
            , boxShadow4 (px 2) (px 2) (px 5) (hex "aaa")
            , borderBottom3 (px 5) solid highlightColor
            , backgroundColor (hex "fff")
            ]


singleValueContainer : List (Attribute msg) -> List (Html msg) -> Html msg
singleValueContainer =
    styled div
        [ displayFlex
        , maxWidth (px 650)
        , flexWrap wrap
        ]

singleValue : String -> Float -> String -> Html msg
singleValue symbol value unit =
    let
        divStyle =
            [ width (px 650)
            , displayFlex
            ]

        spanStyle =
            [ margin2 (px 0) (px 4)
            ]

        mainFontSize = (px 55)

        symbolStyle =
            spanStyle ++
            [ fontSize mainFontSize
            ]

        valueStyle =
            spanStyle ++
            [ fontSize mainFontSize
            ]

        unitStyle =
            spanStyle ++
            [ fontSize (px 25)
            , marginTop (px 7)
            , color (hex "909090")
            ]

    in
        styled div divStyle []
            [ styled span symbolStyle [] [text symbol]
            , styled span valueStyle [] [text <| toString value]
            , styled span unitStyle [] [text unit]
            ]



{-|
  A button with the standard look for the project
-}
styledButton : List (Attribute msg) -> List (Html msg) -> Html msg
styledButton =
    styled button
        [ border (px 0)
        , borderBottom3 (px 4) solid (rgba 0 0 0 0)
        , borderRadius (px 3)
        , padding2 (px 4) (px 6)
        , paddingBottom (px 0)
        , boxShadow4 (px 2) (px 2) (px 2) (hex "aaa")
        , margin (px 3)
        , hover
            [ borderBottomColor (rgb 207 161 227)]
        ]



