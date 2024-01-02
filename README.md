# static-api

This is a simple application simulating a basic REST API. It allows CRUD operations (Create, Read, Update, Delete) on different collections, where each collection is represented as a JSON file in the file system. If the collection does not exist, it is automatically created.

By default, the app will listen on localhost:5800. If you need to change the HOST and PORT, you can either declare the environment variables IPHOST and PORT, or create a .env file with these settings.

## Endpoints

### Get all items in a collection (GET ALL or POST)

```bash
curl -X GET http://localhost:5800/api/<collection>

curl -X POST -H "Content-Type: application/json" -d '{"field1":"value1", "field2":"value2"}' http://localhost:5800/api/<collection>
```

### Get a specific item by ID (GET ONE)
```bash
curl -X GET http://localhost:5800/api/<collection>/<id>
```

### Update a specific item by ID (PUT)

```bash
curl -X PUT -H "Content-Type: application/json" -d '{"field1":"new_value1", "field2":"new_value2"}' http://localhost:5800/api/<collection>/<id>
```

### Delete a specific item by ID (DELETE)

```bash
curl -X DELETE http://localhost:5800/api/<collection>/<id>
```

## Examples
### Create a new item in a collection

```bash
curl -X POST -H "Content-Type: application/json" -d '{"name":"New Item", "value":42}' http://localhost:5800/api/example
```

### Get all items in a collection

```bash
curl -X GET http://localhost:5800/api/example
```

### Get a specific item by ID

```bash
curl -X GET http://localhost:5800/api/example/1
```

### Update a specific item by ID

```bash
curl -X PUT -H "Content-Type: application/json" -d '{"name":"Updated Item", "value":99}' http://localhost:5800/api/example/1
```

### Delete a specific item by ID

```bash
curl -X DELETE http://localhost:5800/api/example/1
```
