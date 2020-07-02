import Http
import Json.Decode as D
import Dict

type BugKind =
       InvalidResponseBody Http.Metadata D.Error -- Failed to decode json. most likely a bug in humblegen
     | InvalidRequestUrl -- probably a bug in the elm url core package

type alias VersionPairing =
  { server: Maybe String
  , client: String
  }

type Error =
      MissingOrInvalidAuth Http.Metadata
    | ServerClientVersionMismatch Http.Metadata VersionPairing
    | OtherwiseBadStatus Http.Metadata -- most likely a server internal runtime error or bug in humblegen
    | NetworkProblems
    | Bug BugKind

type alias Success a = {
    data: a,
    metadata: Http.Metadata
    }

type alias Response a = Result Error (Success a)

requestId : Http.Metadata -> Maybe String
requestId metadata =
    Dict.get "request-id" metadata.headers 

serverVersion : Http.Metadata -> Maybe String
serverVersion metadata =
    Dict.get "backend-version" metadata.headers

{-| Compare client and backend server versions, ignoring dirty worktrees.
-}
eqVersions : VersionPairing -> Bool
eqVersions version =
  let
    server = Maybe.map (String.replace "-modified" "") version.server
    client = String.replace "-modified" "" version.client
  in
  server == Just client

{-| Interpret response of a server

Result is ok if status code was 200. 
-}
expectRestfulJson : (Response a -> msg) -> String -> D.Decoder a -> Http.Expect msg
expectRestfulJson toMsg clientVersion decoder =
  Http.expectStringResponse toMsg <|
    \response ->
      case response of
        Http.BadUrl_ url ->
          Err <| Bug InvalidRequestUrl

        Http.Timeout_ ->
          Err <| NetworkProblems

        Http.NetworkError_ ->
          Err <| NetworkProblems

        Http.BadStatus_ metadata body ->
            -- Body of these responses is JSON with two fields: code and kind. Since
            -- we already know the code, and kind is just server internal stack-trace-like garbage neither
            -- frontend code nor end-user care about, discard...
            let
              maybeServerVersion = (serverVersion metadata)

              versions = VersionPairing maybeServerVersion clientVersion
            in if not (eqVersions versions) && maybeServerVersion /= Nothing then
              Err <| ServerClientVersionMismatch metadata versions
            else if metadata.statusCode == 401 then
              Err <| MissingOrInvalidAuth metadata
            else
              Err <| OtherwiseBadStatus metadata

        Http.GoodStatus_ metadata body ->
          -- TODO: note, we do not error if the http status is good, even though the server and client version mismatch.
          -- not sure if this is a good idea or not? there are advantages and downsides
          case D.decodeString decoder body of
            Ok value ->
              Ok <| Success value metadata

            Err err ->
              let
                versions = VersionPairing (serverVersion metadata) clientVersion
              in if not (eqVersions versions) then
                Err <| ServerClientVersionMismatch metadata versions
              else
                Err <| Bug <| InvalidResponseBody metadata err


-- TODO: this code is obviously not portable to other projects, add auth annotations to humblegen
withAuthorization : String -> Http.Header
withAuthorization session = 
    Http.header "Authorization" <| "CUSTOM " ++ session

maybeWithAuthorization : Maybe String -> List Http.Header
maybeWithAuthorization = 
    (Maybe.map (withAuthorization >> List.singleton)) >> (Maybe.withDefault [])