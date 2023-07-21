/// Конфигурация
pub mod config;
pub mod state;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, http::header::{self, ContentType}, HttpRequest, guard};
use config::AppConfig;
use path_template::PathTemplateParser;
use serde::__private::de::Content;
use std::{io::prelude::*, fs::File};
use std::io;
use std::{env, path::PathBuf, sync::{Arc, Mutex}};
use mime;

use crate::state::AppState;

#[get("/")]
async fn hello( state: web::Data<AppState> ) -> impl Responder {
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

async fn index( req: HttpRequest, state: web::Data<AppState> ) -> impl Responder {
    println!("uri path {}",req.uri().path());

    if req.uri().path().contains("../") || req.uri().path().contains("/..") {
        return HttpResponse::Forbidden().body("can't read this resource");
    }

    let static_files = state.static_files.lock().unwrap();
    if static_files.is_none() {
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

    match File::open(target_file) {
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
            HttpResponse::InternalServerError().body(format!("can't read {}", &err.to_string()))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_conf = AppConfig::find_or_default();
    let app_conf = Arc::new(app_conf);

    println!("starting server on {}:{}", &app_conf.web_server.host, app_conf.web_server.port);

    let conf = app_conf.clone();
    HttpServer::new(move || {
        let template_parser = PathTemplateParser::default()
        .with_variable("exe.dir", env::current_exe().map(|f| f.parent().unwrap().to_str().unwrap().to_string() ).unwrap() )
        .with_variable("work.dir", conf.work_dir.to_string());

        let static_files_opt = conf.web_server.static_files.clone().map(|tmpl|{
            PathBuf::from( template_parser.parse(&tmpl).unwrap().generate() )
        });
        let static_files_opt = Arc::new(Mutex::new(static_files_opt));

        let app = App::new();
        let app = app.app_data(web::Data::new(AppState {
            static_files: static_files_opt
        }));

        let app = app
            .service(web::resource("/{name}.{ext:html|css|js|png|jpg}").route(web::route().guard(guard::Get()).to(index)));
        let app = app.service(hello);
        app
    })
    .bind((app_conf.web_server.host.clone(), app_conf.web_server.port))?
    .run()
    .await
}