import Json.Decode as D
import Json.Encode as E

type Uuid = Uuid String


encode : Uuid -> String
encode (Uuid str) = E.string str

decode : D.Decoder Uuid
decode (Uuid str) = D.map Uuid D.string