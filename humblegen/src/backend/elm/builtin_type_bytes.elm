import Json.Decode as D
import Json.Encode as E

type Bytes = Bytes String


encode : Bytes -> String
encode (Bytes str) = E.string str

decode : D.Decoder Bytes
decode (Bytes str) = D.map Bytes D.string