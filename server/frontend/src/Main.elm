module Main exposing (..) 

import Html exposing (..)
import Time exposing (Posix)
import Svg exposing (..)
import Svg.Attributes exposing (..)
import Dict exposing (Dict)
import Browser.Navigation
import Browser
import Url exposing (Url)

import Time
import Msg exposing (Msg(..))
import Model exposing (Model)
import View exposing (view)
import Requests exposing (sendAvailableDataQuery, sendValueRequest)

import Constants exposing (day)




init : () -> Url -> Browser.Navigation.Key -> (Model, Cmd Msg)
init _ location _ =
    ( { values = Dict.empty
      , listedData = []
      , availableData = []
      , url = serverUrlFromLocation location
      , timeRange = day
      }
    , Cmd.none)



update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        ValuesReceived name values ->
            case values of
                Ok values_ ->
                    let
                        newValues = Dict.insert name values_ model.values
                    in
                        ({model | values = newValues}, Cmd.none)
                Err e ->
                    let
                        _ = Debug.log "Failed to make http request" e
                    in
                        (model, Cmd.none)
        Tick time ->
            let
                requests = sendAvailableDataQuery model.url
                    :: List.map (sendValueRequest model.url) model.listedData
            in
                (model, Cmd.batch requests)
        AvailableDataReceived data ->
            case data of
                Ok availableData ->
                    ({ model 
                        | availableData = availableData
                        , listedData = availableData
                     }
                     , Cmd.none)
                Err e ->
                    let
                        _ = Debug.log "Failed to get available data" e
                    in
                        (model, Cmd.none)
        ToggleData name ->
            let
                (newListed, newValues) =
                    if List.member name model.listedData then
                        ( List.filter (\x -> x /= name) model.listedData
                        , Dict.remove name model.values
                        )
                    else
                        (name :: model.listedData, model.values)
            in
                ({model | listedData = newListed, values = newValues}, Cmd.none)
        UrlChanged location ->
            let
                _ = Debug.log "location.fragment" location.fragment
            in
                ({model | url = serverUrlFromLocation location}, Cmd.none)
        TimeRangeChanged time ->
            ({model | timeRange = time}, Cmd.none)
        Dummy ->
            (model, Cmd.none)




serverUrlFromLocation : Url -> String
serverUrlFromLocation location =
    case location.fragment of
        Just url -> url
        Nothing ->
            let
                p = Maybe.withDefault ""
                    <| Maybe.map ((++) ":")
                    <| Maybe.map String.fromInt
                    <| location.port_
            in
            location.host ++ p

subscriptions : Model -> Sub Msg
subscriptions _ =
    Time.every 1000 Tick


main : Program () Model Msg
main =
    Browser.application
        { init = init
        , update = update
        , view = View.view
        , subscriptions = subscriptions
        , onUrlRequest = \_ -> Dummy
        , onUrlChange = UrlChanged
        }


