use std::{path::PathBuf, fs::File};

use actix_web::{get, web, HttpResponse, Responder, http::header::{self, ContentType}, HttpRequest};
use crate::state::AppState;
use std::io::prelude::*;
use log::{info, debug, error, warn, trace};

/// Переадресут на index.html
#[get("/")]
pub async fn hello<'a>( state: web::Data<AppState> ) -> impl Responder {
    info!("get / endpoint");

    let static_files = state.static_files.lock().unwrap();
    let default = || HttpResponse::Ok().body("Hello world!");

    static_files.clone().map(|static_root| {
        let index_file = static_root.join("index.html");
        if index_file.is_file() {
            HttpResponse::TemporaryRedirect().append_header((header::LOCATION, "/index.html")).body("body")
        } else {
            default()
        }
    }).unwrap_or(default())
}

fn log_request( req: &HttpRequest ) {
    info!("has got request {uri} from {from:?}",uri=req.uri(), from=req.connection_info().peer_addr())
}

/// Чтение статического ресурса (html/css/js/png/jpg)
pub async fn get_static<'a>( req: HttpRequest, state: web::Data<AppState> ) -> impl Responder {
    if req.uri().path().contains("../") || req.uri().path().contains("/..") {
        warn!("forbidden by uri path contains: '../' | '/..' ");
        return HttpResponse::Forbidden().body("can't read this resource");
    }

    let static_files = state.static_files.lock().unwrap();
    if static_files.is_none() {
        warn!("static files root not defined");
        return HttpResponse::NotFound().body("static root not defined");
    }

    let root = static_files.clone().unwrap();
    let path = PathBuf::from(
        {
            if req.uri().path().starts_with("/") && req.uri().path().chars().count()>1 {
                req.uri().path().split_at( req.uri().path().char_indices().skip(1).next().unwrap().0 ).1
            } else {
                req.uri().path()
            }
        }
    );

    let target_file = root.join(path);
    let mut tf = target_file.clone();
    let root_matched = {
        loop {
            match tf.parent() {
                Some(prnt) => {
                    if prnt.to_str().and_then(|p1| root.to_str().map(|p2| p1 == p2)).unwrap_or(false) {
                        break true;
                    } else {
                        tf = prnt.to_path_buf();
                    }
                }
                None => { break false; }
            }
        }
    };

    if !root_matched {
        warn!("forbidden, resource outside of root");
        return HttpResponse::Forbidden().body("can't read this resource");
    }

    let ct = if req.uri().path().ends_with(".html") {
        ContentType::html()
    } else if req.uri().path().ends_with(".css") {
        ContentType(mime::TEXT_CSS)
    } else if req.uri().path().ends_with(".js") {
        ContentType(mime::TEXT_JAVASCRIPT)
    } else if req.uri().path().ends_with(".png") {
        ContentType::png()
    } else if req.uri().path().ends_with(".jpg") {
        ContentType::jpeg()
    } else {
        ContentType::octet_stream()
    };

    match File::open(target_file.clone()) {
        Ok(mut file) => {
            let mut buf : Vec<u8> = vec![];
            match file.read_to_end(&mut buf) {
                Ok(data_size) => {
                    HttpResponse::Ok()
                        .content_type(ct)
                        .append_header(("Content-Length", data_size.to_string()))
                        .body(buf)
                },
                Err(err) => {
                    HttpResponse::InternalServerError().body(format!("can't read {}", &err.to_string()))
                }
            }
        },
        Err(err) => {
            error!("can't read file {:?}", target_file.clone());
            HttpResponse::InternalServerError().body(format!("can't read {}", &err.to_string()))
        }
    }
}

