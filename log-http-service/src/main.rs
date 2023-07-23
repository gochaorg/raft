/// Конфигурация
mod config;

/// Состояние
mod state;

/// Rest api для работы с очередью
mod queue_api;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, http::header::{self, ContentType}, HttpRequest, guard};
use config::AppConfig;
use logs::{logqueue::{find_logs::FsLogFind, LogQueueConf, LogQueueFileNumID, LogQueueFileNumIDOpen, ValidateStub, LogFileQueue}, bbuff::absbuff::FileBuff, logfile::LogFile};
use logs::logqueue::path_template2;
use path_template::PathTemplateParser;
use std::{io::prelude::*, fs::File, marker::PhantomData};
use std::{env, path::PathBuf, sync::{Arc, Mutex}};
use mime;

use crate::state::AppState;

/// Переадресут на index.html
#[get("/")]
async fn hello<'a>( state: web::Data<AppState> ) -> impl Responder {
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

/// Чтение статического ресурса (html/css/js/png/jpg)
async fn index<'a>( req: HttpRequest, state: web::Data<AppState> ) -> impl Responder {
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

/// Очередь
static mut QUEUE_GLOBAL: Option<Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>  >>> = None;

/// Работа с очередю
/// 
/// Аргументы
/// 
/// - `work` - функция получающая ссылку на очередь
pub fn queue<F,R>( work:F ) -> R 
where
    F: Fn( Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>>> ) -> R
{
    let q = unsafe{ QUEUE_GLOBAL.clone().unwrap() };
    work(q)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_conf = AppConfig::find_or_default();
    let app_conf = Arc::new(app_conf);

    println!("starting server on {}:{}", &app_conf.web_server.host, app_conf.web_server.port);
    let conf_t = app_conf.clone();

    fn template_vars<'a>( tp: PathTemplateParser<'a>, conf:Arc<AppConfig> ) -> PathTemplateParser<'a> {
        tp.with_variable("exe.dir", env::current_exe().map(|f| f.parent().unwrap().to_str().unwrap().to_string() ).unwrap() )
          .with_variable("work.dir", conf.work_dir.to_string())
    }

    let template_parser = move || {
        template_vars(PathTemplateParser::default(),conf_t.clone())
    };

    // static files ..........
    println!("configure static files");
    let static_files_opt = app_conf.web_server.static_files.clone().map(|tmpl|{
        PathBuf::from( template_parser().parse(&tmpl).unwrap().generate() )
    });
    let static_files_opt = Arc::new(Mutex::new(static_files_opt));

    // queue ..........
    println!("openning queue");
    let fs_log_find = FsLogFind::new( 
        &template_parser().parse(&app_conf.queue.find.root).unwrap().generate(), 
        "*.binlog", 
        true ).unwrap();

    let log_queue_conf: LogQueueConf<LogQueueFileNumID, PathBuf, FileBuff, _, _, _, _> = {
        let conf = app_conf.clone();
        LogQueueConf {
            find_files: fs_log_find,
            open_log_file: LogQueueFileNumIDOpen,
            validate: ValidateStub,
            new_file: path_template2( &app_conf.queue.new_file.template, move |tp| template_vars(tp, conf.clone())).unwrap(),
            _p: PhantomData.clone(),
        }
    };    

    let queue = log_queue_conf.open().unwrap();
    let queue: Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>  >> = Arc::new(Mutex::new(queue));

    unsafe {
        QUEUE_GLOBAL = Some(queue);
    }

    println!("queue openned");

    // configure atix ...........
    HttpServer::new(move || {
        let app = App::new();
        let app = app.app_data(web::Data::new(AppState {
            static_files: static_files_opt.clone(),
        }));

        let app = app
            .service(web::resource("/{name}.{ext:html|css|js|png|jpg}").route(web::route().guard(guard::Get()).to(index)));
        let app = app.service(hello);
        let app = app.service(web::scope("/queue").configure(queue_api::queue_api_route));
        app
    })
    .bind((app_conf.clone().web_server.host.clone(), app_conf.web_server.port))?
    .run()
    .await
}