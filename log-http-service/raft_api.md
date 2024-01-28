Добавить узел
=========================

```http
POST http://localhost:8080/raft/node/node-a
content-type: application/json

{
    "baseAddress": "http://127.0.0.2:8080"
}
```

Получение списка узлов
=========================
```http
GET http://localhost:8080/raft/node
```

Ответ

    HTTP/1.1 200 OK
    content-length: 51
    connection: close
    content-type: application/json
    vary: Origin, Access-Control-Request-Method, Access-Control-Request-Headers
    date: Sun, 28 Jan 2024 14:52:27 GMT

```json
{
  "node-a": {
    "base_address": "http://localhost:8081"
  }
}
```

Получение списка узлов
=========================
```http
GET http://localhost:8080/raft/node/node-a
```
