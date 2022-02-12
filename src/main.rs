
extern crate juniper;

use std::io;
use std::sync::Arc;

use actix_web::{web, App, Error, HttpResponse, HttpServer};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod schema;
use schema::{create_schema, Schema};

fn main() -> io::Result<()> {
    let schema = std::sync::Arc::new(create_schema()); // Initialize the graphql schema
    HttpServer::new(move || { // move: to create a copy of the schema
        App::new()
            .data(schema.clone()) // clone the schema
            .service(web::resource("/graphql").route(web::post().to_async(graphql))) // service for executing query and mutation requests
            .service(web::resource("/graphiql").route(web::get().to(graphiql))) // service for providing an interface to send the requests
    })
    .bind("localhost:8080")? // start on port 8080
    .run()
}



fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {

    // Get the GraphQL request in JSON

    web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })

    // Error occurred.
    .map_err(Error::from)

    // Successful.
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

fn graphiql() -> HttpResponse {

    // Get the HTML content
    let html = graphiql_source("http://localhost:8080/graphql");
    
    // Render the HTML content

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}