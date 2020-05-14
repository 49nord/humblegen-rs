# Humblegen File Specification

This document describes the humblespec language.

## Data Types

### Built-ins

### Enums

### Structs

#### Embedding

## Doc Comments

## Service Definitions

A service definition defines a set of endpoints.
An endpoint is comprised of
* a **method** (GET, POST)
* a **route** consisting of slash-separated **route components**, which can be
  * a literal route component (kebab-case)
  * a parameter that can be deserialized from a string that does not contain a slash
* an optional **query** type specified by `?{`*`StructType`*`}`
* for `POST` requests, a **body type**
* a **response type**

**Example:**

```
service ServiceName {
    GET     /version -> str,
    GET     /products?{ProductQuery} -> list[Product],
    POST    /product/{id: str}/reviews -> ReviewData -> result[Review][PostReviewError],
}

struct ProductQuery {
    name: option[str],
    price_range: (u32, u32),
}

struct Product { }

enum PostReviewError {
    MaxLengthExceeded(i32),
}

```

### Error Handling

A built-in `ServiceError` type covers all errors that are not specific to the domain model that the service represents and/or provides:
- *authorization* failure (HTTP status code `401`)
- *authentication* failure (HTTP status code `403`)
- *interal* error (e.g. database down) (HTTP status code `500`)

The service error type does not show up in the humblespec service definition for clarity, but users of both client and server code have to deal with it.

**Example**:
The implementor of endpoint
```
    GET /version -> str,
```
will need to return their language's variant of `result[str][ServiceError]`, not just `str`.

The implementor of endpoint

```
    POST    /product/{id: str}/reviews -> ReviewData -> result[Review][PostReviewError],
```

will need to return their language's variant of `result[result[Review][PostReviewError]][ServiceError]`.


### Queries

An endpoint can take an optional query parameter 