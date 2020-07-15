import Date -- justinmimbs/date
import Dict exposing (Dict)
import Iso8601  -- rtfeldman/elm-iso8601-date-strings
import Json.Decode as D
import Time  -- elm/time
import {module_prefix}.BuiltIn.Bytes as BuiltinBytes
import {module_prefix}.BuiltIn.Uuid as BuiltinUuid

-- TODO: move into its own module to avoid name collision

custom : D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
custom =
    D.map2 (|>)

required : String -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
required key valDecoder decoder =
    custom (D.field key valDecoder) decoder

-- A helper function for a required index in a JSON list.
requiredIdx : Int -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
requiredIdx idx itemDecoder decoder =
    custom (D.index idx itemDecoder) decoder

-- Maybe-unwrapping decoder: Turns a `Maybe t` decoder into an a `t` decoder by outputting an error on `Nothing`.
unwrapDecoder : D.Decoder (Maybe t) -> D.Decoder t
unwrapDecoder =
    D.andThen
        (\x ->
            case x of
                Just v ->
                    D.succeed v

                Nothing ->
                    D.fail "invalid enum string value"
        )


builtinDecodeDate : D.Decoder Date.Date
builtinDecodeDate =
    D.map Date.fromIsoString D.string
    |> D.andThen
        (\result ->
            case result of
                Ok v ->
                    D.succeed v

                Err errMsg ->
                    D.fail <| "not a valid date: " ++ errMsg
        )

builtinDecodeIso8601 : D.Decoder Time.Posix
builtinDecodeIso8601 =
    Iso8601.decoder


builtinDecodeResult : D.Decoder error -> D.Decoder value -> D.Decoder (Result error value)
builtinDecodeResult error value =
    D.oneOf 
        [ D.field "Ok" value |> D.map Ok
        , D.field "Err" error |> D.map Err
        ]

builtinDecodeOption : D.Decoder value -> D.Decoder (Maybe value)
builtinDecodeOption =
    D.nullable