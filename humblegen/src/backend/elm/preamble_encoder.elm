import Date -- justinmimbs/date
import Dict exposing (Dict)
import Iso8601  -- rtfeldman/elm-iso8601-date-strings
import Json.Encode as E
import Time  -- elm/time
import Url.Builder


builtinEncodeDate : Date.Date -> E.Value
builtinEncodeDate =
    Date.toIsoString >> E.string


builtinEncodeMaybe : (t -> E.Value) -> Maybe t -> E.Value
builtinEncodeMaybe encoder =
    Maybe.map encoder >> Maybe.withDefault E.null


builtinEncodeResult : (err -> E.Value) -> (ok -> E.Value) -> Result err ok -> E.Value
builtinEncodeResult errEncoder okEncoder res =
    case res of
        Err err -> E.object [("Err", errEncoder err)] 
        Ok ok -> E.object [("Ok", okEncoder ok)]