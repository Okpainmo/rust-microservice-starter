# Rusty Auth Postman Collection

This directory contains the Postman collection for the current Rusty Auth API build.

## Collection

```text
rusty-auth.postman_collection.json
```

The collection name inside Postman is `Rusty Auth API`.

## Import

1. Open Postman.
2. Select `Import`.
3. Choose `postman/rusty-auth.postman_collection.json`.
4. Confirm the import.

## Base URL

The collection uses this default variable:

```text
base_url=http://127.0.0.1:8000/api/v1/auth
```

Update `base_url` in the collection variables if the service is running on a different host or port.

## Auth Flow

Run one of these public requests first:

- `Public Auth / Register User`
- `Public Auth / Register Admin`
- `Public Auth / Login`

Those requests capture the main auth variables from the response:

- `access_token`
- `refresh_token`
- `session_id`
- `user_id`
- `user_email`
- `auth_cookie`

Protected responses can rotate the active credentials. The collection-level test script updates
`access_token`, `refresh_token`, and `session_id` whenever a response includes fresh values, so run
protected requests in Postman as a sequence rather than manually reusing old tokens.

Protected requests use those variables in the required headers:

```http
Authorization: Bearer {{access_token}}
user_id: {{user_id}}
session_token: {{refresh_token}}
session_id: {{session_id}}
Cookie: auth_cookie={{auth_cookie}}
```

## Included Request Groups

- `Public Auth`
- `Sessions`
- `Roles`
- `Permissions`
- `Logout`

Run `Logout / Logout Current Session` last when manually testing a flow, because it revokes the
current backing session.
