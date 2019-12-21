extern crate hyper;
extern crate hyper_staticfile;

mod docs;
pub use self::docs::*;

pub mod markup;

use crate::pkg::ManifestFile;
use colored::Colorize;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Request, Response, Server};

pub fn serve(port: u16, mut docs: docs::Docs) {
    docs.apply_versions(&Versions {
        pkgfile: ManifestFile::new("pkg.yml").load().unwrap(),
        lockfile: ManifestFile::new(".pkg.lock").load().unwrap(),
    });

    let addr = ([127, 0, 0, 1], port).into();

    let new_svc = move || {
        let docs = serde_json::to_string(&docs).unwrap();
        let docs_html = hyper_staticfile::Static::new("/usr/local/lib/loa/docs-html");

        service_fn(move |req| {
            if req.uri().path() == "/docs.json" {
                return Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(docs.clone().into());
            }
            let found = docs_html.serve(req).wait().unwrap();
            if found.status().is_success() {
                return Ok(found);
            }
            Ok(docs_html
                .serve(Request::get("/").body(()).unwrap())
                .wait()
                .unwrap())
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Serving docs on port {}", format!("{}", port).green());

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(format!("http://localhost:{}", port))
            .output()
            .map(|_| ())
            .unwrap_or(());
    }

    hyper::rt::run(server);
}
