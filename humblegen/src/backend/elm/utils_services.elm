{-| A helper function for a required field on a JSON object.
-}
required : String -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
required fieldName itemDecoder functionDecoder =
  D.map2 (|>) (D.field fieldName itemDecoder) functionDecoder

{-| A helper function for a required index in a JSON list.
-}

requiredIdx : Int -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
requiredIdx idx itemDecoder functionDecoder =
    D.map2 (|>) (D.index idx itemDecoder) functionDecoder

{-| Maybe-unwrapping decoder.

Turns a `Maybe t` decoder into an a `t` decoder by outputting an error on `Nothing`.
-}
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

{-| Encode `Maybe` as `null` or value.

-}
encMaybe : (t -> E.Value) -> Maybe t -> E.Value
encMaybe enc = Maybe.withDefault E.null << Maybe.map enc

{-| Decode `Date` from ISO string.
-}
dateDecoder : D.Decoder Date.Date
dateDecoder =
    D.map Date.fromIsoString D.string
    |> D.andThen
        (\result ->
            case result of
                Ok v ->
                    D.succeed v

                Err errMsg ->
                    D.fail <| "not a valid date: " ++ errMsg
        )

{-| Encode `Date` as ISO string.
-}

encDate : Date.Date -> E.Value
encDate = Date.toIsoString >> E.string