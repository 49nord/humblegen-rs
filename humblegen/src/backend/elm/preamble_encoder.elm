import Date -- justinmimbs/date
import Dict exposing (Dict)
import Iso8601  -- rtfeldman/elm-iso8601-date-strings
import Json.Encode as E
import Time  -- elm/time
import Url.Builder
import {module_prefix}.BuiltIn.Bytes as BuiltinBytes
import {module_prefix}.BuiltIn.Uuid as BuiltinUuid




builtinEncodeDate : Date.Date -> E.Value
builtinEncodeDate =
    Date.toIsoString >> E.string

builtinEncodeIso8601 : Time.Posix -> E.Value
builtinEncodeIso8601 =
    Iso8601.encode


builtinEncodeMaybe : (t -> E.Value) -> Maybe t -> E.Value
builtinEncodeMaybe encoder =
    Maybe.map encoder >> Maybe.withDefault E.null


builtinEncodeResult : (err -> E.Value) -> (ok -> E.Value) -> Result err ok -> E.Value
builtinEncodeResult errEncoder okEncoder res =
    case res of
        Err err -> E.object [("Err", errEncoder err)] 
        Ok ok -> E.object [("Ok", okEncoder ok)]