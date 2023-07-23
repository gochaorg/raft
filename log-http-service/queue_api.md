Методы

- Получить список файлов
- Получение rid текущей очереди
- Добавление plain записи
- Получение тела записи
- Просмотр заголовков последних 2 записей

Получить список файлов
==================================

```http
GET http://localhost:8080/queue/log/files
```

ответ

    HTTP/1.1 200 OK
    content-length: 129
    connection: close
    content-type: application/json
    date: Sat, 22 Jul 2023 21:26:29 GMT

    {
        "files": [
            {
            "name": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
            "items_count": 4
            }
        ]
    }

Получение rid текущей очереди
==================================

```http
GET http://localhost:8080/queue/tail/id
```

ответ

    HTTP/1.1 200 OK
    content-length: 27
    connection: close
    content-type: application/json
    date: Sat, 22 Jul 2023 19:23:10 GMT

    {
    "log_id": "0",
    "rec_id": "3"
    }

Просмотр заголовков последних 2 записей
============================================

```http
GET http://localhost:8080/queue/headers/last/2
```

ответ

    HTTP/1.1 200 OK
    content-length: 308
    connection: close
    content-type: application/json
    date: Sat, 22 Jul 2023 19:25:02 GMT

```json
{
  "values": [
    {
      "rid": {
        "log_id": "0",
        "block_id": "3"
      },
      "result": {
        "Succ": {
          "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
          "log_id": "0",
          "block_id": "3",
          "options": {
            "time": "2023-07-21T22:37:38.481669+00:00",
            "mime": "text/plain",
            "encoding": "utf-8"
          },
          "position": "380",
          "head_size": 135,
          "data_size": 7,
          "tail_size": 8
        }
      }
    },
    {
      "rid": {
        "log_id": "0",
        "block_id": "2"
      },
      "result": {
        "Succ": {
          "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
          "log_id": "0",
          "block_id": "2",
          "options": {
            "mime": "text/plain",
            "encoding": "utf-8",
            "time": "2023-07-21T22:29:36.327761+00:00"
          },
          "position": "231",
          "head_size": 135,
          "data_size": 6,
          "tail_size": 8
        }
      }
    }
  ],
  "navigate_error": null
}
```