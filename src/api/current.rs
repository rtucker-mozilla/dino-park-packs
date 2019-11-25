use crate::db::db::Pool;
use crate::db::operations;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;

fn join(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    let user = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    match operations::invitations::accept_invitation(
        &pool,
        &scope_and_user,
        &group_name,
        &user,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Err(error::ErrorNotFound(e)),
    }
}

pub fn current_app() -> impl HttpServiceFactory {
    web::scope("/self")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/join/{group_name}").route(web::post().to(join)))
    
}


