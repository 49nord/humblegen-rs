import Date -- justinmimbs/date
import Dict exposing (Dict)
import Iso8601  -- rtfeldman/elm-iso8601-date-strings
import Json.Encode as E
import Time  -- elm/time

{-| Encode `Date` as ISO string.
-}
encDate : Date.Date -> E.Value
encDate = Date.toIsoString >> E.string

{-| Encode `Maybe` as `null` or value.

-}
encMaybe : (t -> E.Value) -> Maybe t -> E.Value
encMaybe enc = Maybe.withDefault E.null << Maybe.map enc