# static-api

This is a simple application emulating a basic REST API. It allows CRUD operations (Create, Read, Update, Delete) on different collections, where each collection is represented as a JSON file in the file system. If the collection does not exist, it is automatically created.

<img width="1087" height="943" alt="imagen" src="https://github.com/user-attachments/assets/11992c23-d139-458e-a795-8dab59bda0ef" />

This becomes particularly handy during front-end development, especially when the back-end is still in the process of being developed.

By default, the app will listen on `localhost:5800`. Visit `http://localhost:5800` to see the available collections and usage examples.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/<collection>` | List items |
| POST | `/api/<collection>` | Create item |
| GET | `/api/<collection>/<id>` | Get item by ID |
| PUT | `/api/<collection>/<id>` | Replace item |
| PATCH | `/api/<collection>/<id>` | Partial update |
| DELETE | `/api/<collection>/<id>` | Delete item |

### Get all items (GET)

```bash
curl http://localhost:5800/api/<collection>
```

Returns up to 30 items by default. Use `skip` and `limit` to paginate:

```bash
curl "http://localhost:5800/api/<collection>?skip=10&limit=5"
```

#### Filtering

Filter by any field value using query parameters:

```bash
curl "http://localhost:5800/api/<collection>?status=active"
curl "http://localhost:5800/api/<collection>?role=admin&status=active"
```

Supported field types: strings, numbers, booleans.

#### Sorting

Sort results with `_sort` and `_order` (`asc` by default):

```bash
curl "http://localhost:5800/api/<collection>?_sort=name&_order=asc"
curl "http://localhost:5800/api/<collection>?_sort=age&_order=desc"
```

Filters, sorting and pagination can be combined:

```bash
curl "http://localhost:5800/api/<collection>?status=active&_sort=name&skip=0&limit=10"
```

#### Response format

```json
{
  "data": [...],
  "total": 42,
  "limit": 10,
  "skip": 0
}
```

`total` reflects the count after filtering, before pagination.

### Get item by ID (GET)

```bash
curl http://localhost:5800/api/<collection>/<id>
```

### Create item (POST)

If no `id` field is provided, one is generated automatically.

```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"name":"New Item", "value":42}' \
  http://localhost:5800/api/<collection>
```

Returns `201 Created` with the new item.

### Replace item (PUT)

Replaces the entire item with the provided body.

```bash
curl -X PUT -H "Content-Type: application/json" \
  -d '{"name":"Updated Item", "value":99}' \
  http://localhost:5800/api/<collection>/<id>
```

### Partial update (PATCH)

Merges the provided fields into the existing item, leaving the rest unchanged.

```bash
curl -X PATCH -H "Content-Type: application/json" \
  -d '{"status":"inactive"}' \
  http://localhost:5800/api/<collection>/<id>
```

### Delete item (DELETE)

```bash
curl -X DELETE http://localhost:5800/api/<collection>/<id>
```

Returns `204 No Content` on success, `404 Not Found` if the item does not exist.

## Arguments

```
Usage: static-api [OPTIONS]

Options:
  -i, --host <HOST>     IP address of the server [default: 127.0.0.1]
  -p, --port <PORT>     Port that will listen to the server [default: 5800]
  -h, --help            Print help
```

Example:

```bash
./static-api --host 0.0.0.0 --port 5555
```

## Data storage

Collections are stored as JSON files in `~/.static-api/`. Each file is named `<collection>.json` and contains an array of objects.
