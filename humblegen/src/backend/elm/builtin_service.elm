import Http

mapServerResponse : (Result Http.Error a -> msg) -> (Result Http.Error a -> msg)
mapServerResponse x = x


-- TODO: move into its own module to avoid name collision
-- TODO: this code is obviously not portable to other projects, add auth annotations to humblegen
withAuthorization : String -> Http.Header
withAuthorization session = 
    Http.header "Authorization" session

maybeWithAuthorization : Maybe String -> List Http.Header
maybeWithAuthorization = 
    (Maybe.map (withAuthorization >> List.singleton)) >> (Maybe.withDefault [])