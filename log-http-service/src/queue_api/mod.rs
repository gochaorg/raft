use actix_web::web;

mod rec_id;
pub use rec_id::*;

mod info_api;
mod headers_api;
mod log_switch_api;
mod plain_api;

/// настройка ручек
pub fn queue_api_route( cfg: &mut web::ServiceConfig ) {
    cfg
     .service(info_api::get_queue_files)
     .service(info_api::get_cur_id)
     .service(headers_api::lasn_n_headers)
     .service(plain_api::insert_api::insert_plain)
     .service(plain_api::read_api::read_plain)
     .service(log_switch_api::log_switch);
 }
 