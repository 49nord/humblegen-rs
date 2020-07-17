import Json.Decode as D
import Json.Encode as E
import Url.Parser

type Uuid = Uuid String


encode : Uuid -> E.Value
encode (Uuid str) = E.string str

decode : D.Decoder Uuid
decode = D.map Uuid D.string

encodeQuery : Uuid -> String
encodeQuery (Uuid str) = str

encodeUrlcomponent : Uuid -> String
encodeUrlcomponent (Uuid str) = str

toString : Uuid -> String
toString (Uuid str) = str

parseUrl : Url.Parser.Parser (Uuid -> b) b
parseUrl =
    Url.Parser.custom "uuid" (Just << Uuid)
