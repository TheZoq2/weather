module ValueDisplay exposing (valueBox)

drawValueBox : String -> List Float -> Html a
drawValueBox icon data =
    let
        min = List.min data
        max = List.max data
        mean = (List.sum data) / (List.length data)
    in
        div [] []


