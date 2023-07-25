Методы

- Получить список файлов
- Получение rid текущей очереди
- Просмотр заголовков последних 2 записей
- Добавление plain записи
- Чтение содержимого записи
- Чтение raw данных записи
- Запись raw данных записи
- Переключение лог файла

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

```json
{
  "files": [
    {
      "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
      "items_count": 8,
      "bytes_count": 1138
    },
    {
      "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-26T03-06-37-qadnb.binlog",
      "items_count": 1,
      "bytes_count": 118
    }
  ]
}
```

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
            "mime": "text/plain",
            "encoding": "utf-8",
            "time": "2023-07-21T22:37:38.481669+00:00"
          },
          "position": "380",
          "head_size": 135,
          "data_size": 7,
          "tail_size": 8,
          "preview": "bla bla"
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
            "time": "2023-07-21T22:29:36.327761+00:00",
            "encoding": "utf-8",
            "mime": "text/plain"
          },
          "position": "231",
          "head_size": 135,
          "data_size": 6,
          "tail_size": 8,
          "preview": "hello2"
        }
      }
    }
  ]
}
```


Добавление plain записи
==========================

```http
POST http://localhost:8080/queue/insert/plain HTTP/1.1
content-type: text/plain

sample data
```

ответ

    HTTP/1.1 200 OK
    content-length: 29
    connection: close
    content-type: application/json
    date: Sun, 23 Jul 2023 19:14:20 GMT

```json
{
  "log_id": "0",
  "block_id": "4"
}
```

Чтение содержимого записи
===========================

```http
GET http://localhost:8080/queue/record/0/4/plain?opt2head=true&opt_prefix=bl_ HTTP/1.1
```

ответ

    HTTP/1.1 200 OK
    transfer-encoding: chunked
    connection: close
    content-type: text/plain
    bl_mime: text/plain
    bl_time: 2023-07-23T19:14:21.027414+00:00
    bl_encoding: utf-8
    date: Sun, 23 Jul 2023 19:23:40 GMT

    sample data


Чтение raw данных записи
=================================

    🚀 curl -v http://localhost:8080/queue/record/0/4/raw > data
    *   Trying 127.0.0.1:8080...
    % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                    Dload  Upload   Total   Spent    Left  Speed
    0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0* Connected to localhost (127.0.0.1) port 8080 (#0)
    > GET /queue/record/0/4/raw HTTP/1.1
    > Host: localhost:8080
    > User-Agent: curl/7.81.0
    > Accept: */*
    > 
    * Mark bundle as not supporting multiuse
    < HTTP/1.1 200 OK
    < transfer-encoding: chunked
    < content-type: application/octet-stream
    < cache-control: max-age=86400
    < date: Sun, 23 Jul 2023 19:27:50 GMT
    < 
    { [177 bytes data]
    100   166    0   166    0     0  87737      0 --:--:-- --:--:-- --:--:--  162k

Данные

    🚀 hexdump -C data 
    00000000  93 00 00 00 0b 00 00 00  08 00 04 00 00 00 00 04  |................|
    00000010  00 00 03 00 00 00 03 00  00 00 7c 01 00 00 00 00  |..........|.....|
    00000020  00 00 03 00 00 00 7c 01  00 00 00 00 00 00 03 00  |......|.........|
    00000030  00 00 7c 01 00 00 00 00  00 00 03 00 00 00 00 00  |..|.............|
    00000040  00 00 04 00 6d 69 6d 65  0a 00 00 00 74 65 78 74  |....mime....text|
    00000050  2f 70 6c 61 69 6e 08 00  65 6e 63 6f 64 69 6e 67  |/plain..encoding|
    00000060  05 00 00 00 75 74 66 2d  38 04 00 74 69 6d 65 20  |....utf-8..time |
    00000070  00 00 00 32 30 32 33 2d  30 37 2d 32 33 54 31 39  |...2023-07-23T19|
    00000080  3a 31 34 3a 32 31 2e 30  32 37 34 31 34 2b 30 30  |:14:21.027414+00|
    00000090  3a 30 30 73 61 6d 70 6c  65 20 64 61 74 61 54 41  |:00sample dataTA|
    000000a0  49 4c a6 00 00 00                                 |IL....|
    000000a6

Запись raw данных записи
====================================

    🚀 curl -v --data-binary @data -X POST http://localhost:8080/queue/record/0/4/raw
    Note: Unnecessary use of -X or --request, POST is already inferred.
    *   Trying 127.0.0.1:8080...
    * Connected to localhost (127.0.0.1) port 8080 (#0)
    > POST /queue/record/0/4/raw HTTP/1.1
    > Host: localhost:8080
    > User-Agent: curl/7.81.0
    > Accept: */*
    > Content-Length: 166
    > Content-Type: application/x-www-form-urlencoded
    > 
    * Mark bundle as not supporting multiuse
    < HTTP/1.1 200 OK
    < content-length: 29
    < content-type: application/json
    < date: Sun, 23 Jul 2023 21:16:32 GMT
    < 
    * Connection #0 to host localhost left intact
    {"log_id":"0","block_id":"5"}

Переключение лог файла
==================================

```http
POST http://localhost:8080/queue/tail/switch HTTP/1.1
```

HTTP/1.1 200 OK
content-length: 118
connection: close
content-type: application/json
date: Tue, 25 Jul 2023 22:06:37 GMT

```json
{
  "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-26T03-06-37-qadnb.binlog",
  "log_id": "1"
}
```