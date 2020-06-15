# Service Protocol Specification

This document describes how humblespec services are mapped to HTTP.

## URL routes

## Regular Responses + Domain Errors

* Responses are encoded as JSON (see `data_types_json_representation.md`).
* HTTP Status code is 200.

* Since **domain errors** are returned as regular response types (e.g. `result[str][GetVersionError]`) by handlers, they **also have status code 200**.

## All Other Errors ("Error Response")

Apart from domain errors (covered in the previous section), the following kinds of errors can happen while handling a request

* **Service-level errors** (auth{n,z}, DB down)
* **Runtime errors**  have a 4XX or 5XX status code.
* **HTTP-level errors** returned by something outside of the control of humblegen (e.g. a misconfigured reverse proxy, etc)
* **DNS + Socket-level errors** caused by
  * something on the network path between client and server
  * a crash of the server (most OSes RST the TCP connection)

The service protocol only addresses **service-level and runtime errors**.
They are represented as status codes 4XX / 5XX and have `content-type: application/json`.

The representation adheres to the following "schema", which is basically what the Rust `serde_json` crate produces for the structures defined in `service_protocol.rs`.

```js
{
    "code": 401, // repetition of the non-2XX HTTP status code
    "kind": {
        // exactly one of the following lines

        "Service": "Authentication",
        "Service": "Authorization",
        "Service": { "Internal": "..." },

        "Runtime": "NoServiceMounted",
        "Runtime": "ServiceMountsAmbiguous",
        "Runtime": { "NoRouteMountedInService": { "service": "..." } },
        "Runtime": { "RouteMountsAmbiguous":    { "service": "..."  } },
        "Runtime": { "RouteParamInvalid": { "param_name": "ROUTE_PARAM_NAME", "parse_error": "..." } },
        "Runtime": { "QueryInvalid": "..." },
        "Runtime": { "PostBodyReadError": "..." },
        "Runtime": { "PostBodyInvalid": "..." }
        "Runtime": { "SerializeHandlerResponse": "..." },
        "Runtime": { "SerializeErrorResponse": "..." },
    }
}
```

