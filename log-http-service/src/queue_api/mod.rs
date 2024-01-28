use actix_web::web;

mod rec_id;
pub use rec_id::*;

/// API Информация о текущей очереди
mod info_api;

/// API Просмотра заголовков
mod headers_api;

/// API Переключение логов
mod log_switch_api;

/// API Чтения / Записи plain/text
mod plain_api;

// API Чтения / Записи binary block
mod raw_api;

/// API Просмотра версии
mod ver_api;

mod err_api;
pub use err_api::*;

/// настройка ручек
pub fn queue_api_route( cfg: &mut web::ServiceConfig ) {
    cfg
     .service(info_api::get_queue_files)
     .service(info_api::get_cur_id)
     .service(headers_api::lasn_n_headers)
     .service(plain_api::insert_plain)
     .service(plain_api::read_plain)
     .service(raw_api::read_block)
     .service(raw_api::write_block)
     .service(ver_api::get_version)
     .service(ver_api::post_version_delay)
     .service(ver_api::get_version_delay)
     .service(log_switch_api::log_switch);
 }
 