import Http
import Json.Decode as D
import Dict

type BugKind =
       InvalidResponseBody Http.Metadata D.Error -- Failed to decode json. most likely a bug in humblegen
     | InvalidRequestUrl -- probably a bug in the elm url core package
     | LogicError -- bug in humblegen server implementation, made possible by lose typing in the humblespec, e.g. if a single error object is
                  -- used for all responses and the error returned does not make sense for the request

type Error =
      MissingOrInvalidAuth Http.Metadata
    | OtherwiseBadStatus Http.Metadata -- most likely a server internal runtime error or bug in humblegen
    | NetworkProblems
    | Bug BugKind

type alias Success a = { -- rust format string escaping
    data: a,
    metadata: Http.Metadata
    } -- rust format string escaping

type alias Response a = Result Error (Success a)

requestId : Http.Metadata -> Maybe String
requestId metadata =
    Dict.get "request-id" metadata.headers 

-- Interpret response of a server
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
            if List.member metadata.statusCode [401, 403] then
              Err <| MissingOrInvalidAuth metadata
            else
              Err <| OtherwiseBadStatus metadata

        Http.GoodStatus_ metadata body ->
          case D.decodeString decoder body of
            Ok value ->
              Ok <| Success value metadata

            Err err ->
              Err <| Bug <| InvalidResponseBody metadata err


-- TODO: this code is obviously not portable to other projects, add auth annotations to humblegen
withAuthorization : String -> Http.Header
withAuthorization session = 
    Http.header "Authorization" <| "Custom " ++ session

maybeWithAuthorization : Maybe String -> List Http.Header
maybeWithAuthorization = 
    (Maybe.map (withAuthorization >> List.singleton)) >> (Maybe.withDefault [])