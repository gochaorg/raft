use actix_web::web;

/// Конфигурация RAFT
pub mod configuration;
pub mod sync;

/*
Create - POST
Read - GET
Update - PUT
Delete - DELETE
*/

/// Настройка маршрутов
pub fn route( cfg: &mut web::ServiceConfig ) {
    use configuration::*;
    use sync::*;

    cfg
        .service(status)
        .service(bg_job_stop)
        .service(bg_job_start)
        .service(node_add)
        .service(node_del)
        .service(node_list)
        .service(node_status)
        .service(master_set)
        .service(master_reset)
        .service(id_get)
        .service(id_set)
        .service(log_shipping_start)
        .service(log_shipping_state)
        ;
}


