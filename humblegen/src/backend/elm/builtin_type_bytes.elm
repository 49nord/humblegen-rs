import Json.Decode as D
import Json.Encode as E
import Bytes as PkgBytes
import Base64 as PkgBase64

type Bytes = Bytes PkgBytes.Bytes

encode : Bytes -> E.Value
encode (Bytes bytes) = E.string <| Maybe.withDefault "" (PkgBase64.fromBytes bytes) -- base64 _en_coding never fails

base64decodeHelper : String  -> D.Decoder Bytes
base64decodeHelper base64Str = case PkgBase64.toBytes base64Str of
    Just bytes ->
        D.succeed (Bytes bytes)
    Nothing ->
        D.fail "invalid base64"

decode : D.Decoder Bytes
decode = D.andThen base64decodeHelper D.string

encodeQuery : Bytes -> String
encodeQuery (Bytes bytes) = PkgBase64.fromBytes bytes |> Maybe.withDefault "" -- base64 _en_coding never fails

encodeUrlcomponent : Bytes -> String
encodeUrlcomponent (Bytes bytes) = PkgBase64.fromBytes bytes |> Maybe.withDefault "" -- base64 _en_coding never fails
