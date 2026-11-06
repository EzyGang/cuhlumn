# Errors

Every failure of an `/api` route returns a JSON error body.
This includes malformed requests, unknown paths, and wrong methods.

## Error body

```json
{
  "error": {
    "code": "validation",
    "message": "brand.accent: the accent \"red\" is not a color: use #rgb or #rrggbb, for example #2563eb",
  }
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `code` | String | A stable error code from the table below |
| `message` | String | A message for people |
| `field` | String or `null` | The JSON path of the invalid value; set only for `validation` errors |

## Codes

| Code | Status | Cause |
| --- | --- | --- |
| `bad_request` | `400` | The body is not valid JSON, or a path parameter cannot be decoded |
| `validation` | `400` | A value breaks a rule, or a database check fails, such as a missing home page |
| `unauthorized` | `401` | No credential, a wrong token, or a missing or expired session |
| `forbidden` | `403` | A cookie request that changes data came from another origin |
| `not_found` | `404` | The site, post, media file, or API path does not exist |
| `method_not_allowed` | `405` | The path does not accept this method |
| `conflict` | `409` | A domain belongs to another site, or the change would break the home page |
| `payload_too_large` | `413` | The body is larger than the limit |
| `unsupported_media_type` | `415` | The body is not JSON on a JSON route, or the image type is not accepted |
| `api_disabled` | `503` | The server has no `CUHLUMN_TOKEN` |
| `internal` | `500` | A server failure; the details go to the server log |

The type `ErrorCode` also contains `console_unavailable`.
The server does not send this code in an API response at this time.

## Field paths

The `field` value uses dots for objects and brackets for arrays:

- `title`
- `brand.accent`
- `brand.links[0].url`
- `tags`
- `host`, `slug`, or `name` for path parameters

For a missing required field, `field` contains the name of that field.
For an unknown field, `field` contains the unknown name.
When `field` is set, `message` starts with the same path and a colon.

## Conflicts

| Request | Cause |
| --- | --- |
| `PUT /api/sites/{host}` | `{host}` is an alias of another site |
| `PUT /api/sites/{host}` | An alias is the domain or an alias of another site |
| `PUT /api/sites/{host}/posts/{slug}` | The post is the home page, and the new `kind` is not `page` |
| `DELETE /api/sites/{host}/posts/{slug}` | The post is the home page |

## Internal errors

For `internal`, the message is always `the server failed; try again later`.
The server writes the cause to its log with `tracing`.
