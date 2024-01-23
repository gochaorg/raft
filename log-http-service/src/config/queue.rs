use serde::{Deserialize, Serialize};

/// Настройки очереди
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Где искать лог файлы
    pub find : QueueFind,

    /// Создание нового лог файла
    pub new_file: QueueNewFile,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self { 
            find: QueueFind::default(), 
            new_file: QueueNewFile::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueFind {
    /// Корневой каталог лог файлов
    pub root: String,

    /// Шаблон имени лог файла
    pub wildcard: String,

    /// Рекурсивный поиск
    pub recursive: bool
}

impl Default for QueueFind {
    fn default() -> Self {
        Self { 
            root: "${work.dir}/app_data/queue".to_string(),
            wildcard: "*.binlog".to_string(), 
            recursive: true 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueNewFile {
    /// Шаблон имени (пути) созданя лог файла
    /// 
    /// Пример
    /// 
    ///     ${work.dir}/app_data/queue/${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog
    /// 
    /// Переменные
    /// -------------------
    /// 
    /// - `${exe.dir}` - путь к каталогу текущего exe файла
    /// - `${work.dir}` - путь к текущему каталогу
    /// - `${....}` - некие переменные которые могут содержать значения
    /// - синтаксис шаблона описан в структуре [TemplateParser]
    /// - `${root}` - это внешняя переменная и должна быть определена явно [with_variable()]
    /// - `${time:...}` - встроенная переменаая, задает текущую дату, формат даты описан в [DateFormat]
    ///     - Синтаксис `[ time_zone ] { qouted | non-quoted | variable }`
    /// - `${rnd:5}` - случайны набор из 5 букв, число 5 - указывает на кол-во букв и может быть заменено на другое число
    /// - `${env:...}` - в качестве значения - потенциально опасно
    /// 
    /// Переменные времени
    /// ---------------------
    /// 
    /// ```
    /// [ time_zone ] { qouted | non-quoted | variable }
    /// ```
    /// 
    /// time_zone - указывает временную зону, возможно три формата
    /// - `utc:` - зона UTC
    /// - `local:` - локальная зона
    /// - `offset+hhmi` либо `offset-hhmi` - смещение относительно UTC, hh - час, mi - минута
    /// 
    /// qouted - строка символов в одиночных ковычках
    /// non-quoted - строка символов, которые не совпадают с variable
    /// variable - переменная
    /// 
    /// По поводу строк
    /// 
    /// - строка начинается с одинарной кавыички
    /// - две подряд кавычки заменяются на одну
    /// 
    /// Примеры:
    /// - например шаблон `yyyy'yyyy'` - будет выведено значение 2023yyyy
    /// - шаблон `mm''dd'hello''mi'mi` - будет `03'25hello'mi34`
    /// 
    /// По поводу переменных
    /// 
    /// - выбирается наиболее длинное совпадение,
    /// 
    /// например между `yyyy` или `yy` - будет выбрано `yyyy`
    /// 
    /// 
    /// | Переменная | Значение                                     | DateTimeItem    | Пример |
    /// |------------|----------------------------------------------|-----------------|--------|
    /// | yyyy       | год - 4 цифры                                | Year            | 1998   | 
    /// | yy         | год - 2 цифры                                | Year2digit      | 98     |
    /// | mm         | месяц 01..12                                 | Month           |
    /// | mmm        | месяц 3 буквы                                | MonthNameShort  |
    /// | mmmm       | месяц полное название                        | MonthNameFull   |
    /// | dd         | дата 01..31                                  | Date            |
    /// | wd         | день недели 3 буквы                          | WeekDayShort    |
    /// | wd0        | день недели Воскресенье=0 ... Суббота=6      | WeekDayZero     |
    /// | wd1        | день недели Понедельник=1 ... Воскресенье=7  | WeekDayOne      |
    /// | ww         | неделя 00..53                                | Week            |
    /// | wwd        | день недели - полное имя                     | WeekDayFull     |
    /// | w1         | неделя 00..53 - неделя начинается с ПН       | WeekMondayFirst |
    /// | hh         | час 0-23                                     | Hour            |
    /// | hp         | час 0-12                                     | Hour12          |
    /// | ha         | час 0-12                                     | Hour12          |
    /// | am         | am или pm                                    | AMPMLoCase      |
    /// | AM         | AM или PM                                    | AMPMHiCase      |
    /// | pm         | am или pm                                    | AMPMLoCase      |
    /// | PM         | AM или PM                                    | AMPMHiCase      |
    /// | mi         | минуты                                       | Minute          |
    /// | ss         | секунды                                      | Second          |
    /// | s3         | миллисек                                     | Millisec        | 026 |
    /// | s6         | микросек                                     | Microsec        | 026490 |
    /// | s9         | наносек                                      | Nanosec         | 026490000 |
    /// | ms         | миллисек                                     | Millisec        | 026490 |
    /// | ns         | наносек                                      | Nanosec         | 026490000 |
    /// | z4         | смещение UTC                                 | Zone4           | +0930 |
    /// | zh         | смещение UTC                                 | ZoneHour        | +09 |
    /// | zhm        | смещение UTC                                 | ZoneHourMin     | +09:30 |
    /// | zhms       | смещение UTC                                 | ZoneHourMinSec  | +09:30:00 |
    /// 
    pub template: String
}

impl Default for QueueNewFile {
    fn default() -> Self {
        Self { 
            template: "${work.dir}/app_data/queue/${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog".to_string() 
        }
    }
}
