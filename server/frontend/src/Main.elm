module Main exposing (..) 

import Html exposing (..)
import Html.Events
import Time exposing (Time, second)
import Svg
import Svg exposing (..)
import Svg.Attributes exposing (..)
import Dict exposing (Dict)
import List.Extra
import Navigation
import Style

import Graph
import Time
import Msg exposing (Msg(..))
import Model exposing (Model)
import View exposing (view)
import Requests exposing (sendAvailableDataQuery, sendValueRequest)

import Constants exposing (day)




init : Navigation.Location -> (Model, Cmd Msg)
init location =
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
                requests = sendAvailableDataQuery model.url
                    :: (List.map (sendValueRequest model.url) model.listedData)
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
                _ = Debug.log "location.hash" location.hash
            in
                ({model | url = serverUrlFromLocation location}, Cmd.none)
        TimeRangeChanged time ->
            ({model | timeRange = time}, Cmd.none)




serverUrlFromLocation : Navigation.Location -> String
serverUrlFromLocation location =
    case String.uncons location.hash of
        Just (hash, url) -> url
        Nothing -> location.host

subscriptions : Model -> Sub Msg
subscriptions model =
    Time.every second Tick


main : Program Never Model Msg
main =
    Navigation.program
        UrlChanged
        { init = init
        , update = update
        , view = View.view
        , subscriptions = subscriptions
        }


