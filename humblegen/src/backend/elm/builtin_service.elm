import Http
import Json.Decode as D
import Json.Encode as E
import Url
import Url.Builder
import Task exposing (Task)

type alias QueryEncoder q = (q -> List Url.Builder.QueryParameter)

type alias Request q t =
    { method : String
    , headers : List Http.Header
    , urlComponents : List String
    , query: Maybe q
    , queryEncoder: QueryEncoder q
    , body : Http.Body
    , resolver : Http.Resolver Error t
    , timeout : Maybe Float
    , base : String
    }


type ResponseBody
    = StringResponse String


type Error
    = Bug String
    | HttpBug Http.Metadata ResponseBody
    | InvalidResponse Http.Metadata ResponseBody D.Error
    | TransportError String
    | AuthorizationError -- humble service protocol level authorization error (e.g. the server-side request handler indicates that the client is unauthorized to access the resource. The client's access token is valid, though.
    | AuthenticationError -- humble service protocol level authentication error (e.g. the server-side request handler indicates that the client did not provide a valid access token)
    | ServerError


makeRequest : String -> List String -> QueryEncoder q -> Http.Resolver Error t -> Request q t
makeRequest method urlComponents queryEncoder resolver =
    { method = method
    , headers = []
    , base = ""
    , query = Nothing
    , queryEncoder = queryEncoder
    , urlComponents = urlComponents
    , body = Http.emptyBody
    , resolver = resolver
    , timeout = Nothing
    }

type alias NoQuery = Never

noQueryEncoder : QueryEncoder Never
noQueryEncoder _ = []

jsonResolver : D.Decoder t -> Http.Resolver Error t
jsonResolver =
    let
        resolve decoder response =
            case response of
                Http.BadUrl_ badUrl ->
                    Err <| Bug <| "bad url: " ++ badUrl

                Http.Timeout_ ->
                    Err <| TransportError "Http.Timeout_"

                Http.NetworkError_ ->
                    Err <| TransportError "Http.NetworkError_"

                Http.BadStatus_ metadata body ->
                    Err <|
                        case metadata.statusCode of
                            401 ->
                                AuthorizationError

                            403 ->
                                AuthenticationError

                            500 ->
                                ServerError

                            _ ->
                                HttpBug metadata (StringResponse body)

                Http.GoodStatus_ metadata body ->
                    D.decodeString decoder body
                        |> Result.mapError (InvalidResponse metadata (StringResponse body))
    in
    Http.stringResolver << resolve


withBase : String -> Request q t -> Request q t
withBase base req =
    { req | base = base }

withQuery : q -> Request q t -> Request q t
withQuery query req =
    { req | query = Just query }

    

withBody : Http.Body -> Request q t -> Request q t
withBody body req =
    { req | body = body }


withTimeout : Float -> Request q t -> Request q t
withTimeout timeout req =
    { req | timeout = Just timeout }


withHeader : String -> String -> Request q t -> Request q t
withHeader name value req =
    { req | headers = Http.header name value :: req.headers }


withJsonBody : (body -> E.Value) -> body -> Request q t -> Request q t
withJsonBody encoder value req =
    { req | body = Http.stringBody "application/json" <| E.encode 2 (encoder value) }


makeUrl : Request q t -> String
makeUrl req =
    Url.Builder.crossOrigin
         req.base
            req.urlComponents
            (Maybe.withDefault [] <| Maybe.map req.queryEncoder req.query)


toTask : Request q t -> Task Error t
toTask req =
    Http.task
        { method = req.method
        , headers = req.headers
        , url = makeUrl req
        , body = req.body
        , resolver = req.resolver
        , timeout = req.timeout
        }
