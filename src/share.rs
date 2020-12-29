use futures::Future;
use warp::Filter;

pub fn lossy_file_name(file: &warp::fs::File) -> Option<String> {
    let file_name = file
        .path()
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    Some(file_name.into())
}

pub fn share_file(path: String, id: String) -> impl Future<Output = ()> {
    pretty_env_logger::init();

    //println!("Serving file {} under /{}", path, id);
    let add_headers = |file: warp::filters::fs::File| {
        let filename = lossy_file_name(&file).unwrap_or_else(|| {
            println!("Couldn't get filename");
            "".into()
        });
        warp::reply::with_header(
            file,
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
    };
    let examples = warp::path(id).and(warp::fs::file(path)).map(add_headers);

    // GET /{id}... => {file}
    let routes = examples;

    //dbg!("Starting http server");

    warp::serve(routes).run(([127, 0, 0, 1], 8080))
}
