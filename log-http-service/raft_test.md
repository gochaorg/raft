Тестирование Raft
===============================

Запустить второй экземпляр `run_second.sh`

Удаление ранее существующего узла

```http
DELETE http://localhost:8080/raft/node/node-a
```

Добавление экземпляра

```http
POST http://localhost:8080/raft/node/node-a
content-type: application/json

{
    "baseAddress": "http://127.0.0.2:8080"
}
```

Получение списка узлов

```http
GET http://localhost:8080/raft/node
```

Ожидаемо

```json
{
  "node-a": {
    "base_address": "http://127.0.0.2:8080",
    "heartbeat_timeout": {
      "secs": 3,
      "nanos": 0
    }
  }
}    
```

Просмотр heart beat

```http
GET http://localhost:8080/raft/node/node-a
```

Установка второго экземпляра - прием только от master

```http
POST http://127.0.0.2:8080/raft/master/set
content-type: application/json

{
    "id": "node-a",
    "base_address": "http://127.0.0.1:8080"
}
```

Просмотр указанного значения

```http
GET http://127.0.0.2:8080/raft/status
```

    HTTP/1.1 200 OK
    content-length: 148
    connection: close
    vary: Origin, Access-Control-Request-Method, Access-Control-Request-Headers
    content-type: application/json
    date: Fri, 28 Jun 2024 14:51:18 GMT

    {
        "id": "node-1",
        "status": "ok",
        "bgJob": {
            "running": true,
            "timeout": {
            "secs": 2,
            "nanos": 0
            }
        },
        "master": {
            "id": "node0",
            "base_address": "http://127.0.0.1:8080"
        }
    }

Проверка отбивки

    🚀 curl --max-redirs 0 -v -X POST http://127.0.0.2:8080/queue/insert/text_plain -d 'xtzy'
    Note: Unnecessary use of -X or --request, POST is already inferred.
    *   Trying 127.0.0.2:8080...
    * Connected to 127.0.0.2 (127.0.0.2) port 8080 (#0)
    > POST /queue/insert/text_plain HTTP/1.1
    > Host: 127.0.0.2:8080
    > User-Agent: curl/7.81.0
    > Accept: */*
    > Content-Length: 4
    > Content-Type: application/x-www-form-urlencoded
    > 
    * Mark bundle as not supporting multiuse
    < HTTP/1.1 307 Temporary Redirect
    < content-length: 23
    < vary: Origin, Access-Control-Request-Method, Access-Control-Request-Headers
    < location: http://127.0.0.1:8080/queue/insert/text_plain
    < date: Fri, 28 Jun 2024 14:55:02 GMT
    < 
    * Connection #0 to host 127.0.0.2 left intact
    accept only from master

Log shipping
=========================

Добавить данные в master, так чтоб бы было отличие, master опережал

```http
POST http://localhost:8080/queue/insert/text_plain
content-type: text/plain

sample data
```

Проверить tail id, на master

```http
GET http://localhost:8080/queue/tail/id
```

    {
        "log_id": "2",
        "block_id": "3"
    }

```http
GET http://127.0.0.2:8080/queue/tail/id
```

    {
        "log_id": "2",
        "block_id": "0"
    }

Запустить транспортировку

```http
POST http://localhost:8080/raft/logShipping/node-a
```

должно быть

```json
{
  "job_id": 0,
  "cargo_size": 4
}
```

Запрос лога

```http
GET http://localhost:8080/raft/logShipping/node-a/0/log
```