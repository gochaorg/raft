Справка о программе
=========================

Синтаксис

    > log-tools { опции } комманда { комманда }

```
args ::= exe_name {command_or_option} 
```

exe_name - первый аргумент - имя exe файла

```
command_or_option ::= option | command
```

Опции влияют на поведение комманд.
Команды выполняют конкретные действия.

```
option ::= verbose | compute_sha256_entry | block_buffer_size | tag

verbose ::= '-v' | '+v'
compute_sha256_entry ::= '-sha256' | '+sha256'
block_buffer_size ::= ( '-block_buffer_size' | '-bbsize' ) ByteSize
tag ::= 'tag' tag_action
tag_action ::= 'clear' | 'default' | tag_add
tag_add ::= ( 'add' | '+' ) tag_key tag_value

ByteSize ::= dec_number {white_space} [size_suffix]
size_suffix ::= kb | mb | gb
kb ::= ( 'K' | 'k' ) b
mb ::= ( 'M' | 'm' ) b
gb ::= ( 'G' | 'g' ) b
b = 'B' | 'b'

command ::= append_cmd | view_cmd | extract_cmd
```

Комманды
- append_cmd - добавляет запись в лог
- view_cmd - просмотр заголовков записей в логе
- extract_cmd - извлечение записи из лога

```
append_cmd ::= ( 'a' | 'append' ) log_file_name append_what
append_what ::= append_file | append_stdin
append_file ::= 'file' append_file_name
append_stdin ::= 'stdin'
```

- log_file_name - имя лог файла

```
view_cmd ::= ( 'v' | 'view' ) log_file_name

extract_cmd ::= ( 'e' | 'extract' ) log_file_name extract_selection
```

- extract_selection - Указывает какие записи необходимо получить

```
extract_selection ::= 'all' | 'range' range_select

range_select ::= range_string_arg
```

- range_string_arg - это параметр коммандной строки, один параметр
 если параметр должен содержать пробел, тогда параметр должен быть в кавычках

```
range_string_arg ::= multiple 

multiple ::= RangeNum { delim RangeNum }
RangeNum ::= FromTo | Single
delim ::= [ WhiteSpace ] ','

FromTo ::= Single [ WhiteSpace ] '-' Single

Single ::= [ WhiteSpace ] Number

Number ::= hex_number | oct_number | bin_number | dec_number
hex_number ::= '0x' hex_digit { hex_digit }
hex_digit  ::= '0' | '1' | '2' | '3' | '4'
             | '5' | '6' | '7' | '8' | '9'
             | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
             | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'

oct_number ::= '0o' oct_digit { oct_digit }
oct_digit  ::= '0' | '1' | '2' | '3' | '4'
             | '5' | '6' | '7'

bin_number ::= '0b' bin_digit { bin_digit }
bin_digit  ::= '0' | '1'

dec_number ::= dec_digit dec_digit
dec_digit  ::= '0' | '1' | '2' | '3' | '4'
             | '5' | '6' | '7' | '8' | '9'
```

