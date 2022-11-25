use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{Body, Method, Request, Response, Server};
use hyper::client::{Client, HttpConnector};
use hyper::service::{make_service_fn, service_fn};
use hyper_tls::HttpsConnector;

async fn get_avatar_url(client: &Client<HttpsConnector<HttpConnector>>, token: &str, user_id: u64) -> anyhow::Result<String> {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("https://discord.com/api/v10/users/{}", user_id))
        .header("accept", "application/json")
        .header("authorization", format!("Bot {}", token))
        .body(Body::empty())?;
    let mut x = client.request(request).await?;
    let body = hyper::body::to_bytes(x.body_mut()).await?;
    let json_data = String::from_utf8(Vec::from(body))?;
    let json: serde_json::Value = serde_json::from_str(&json_data)?;
    let avatar = json["avatar"].as_str().ok_or(anyhow::anyhow!("Missing avatar"))?;
    let id = json["id"].as_str().ok_or(anyhow::anyhow!("Missing avatar"))?;
    let avatar_url = format!("https://cdn.discordapp.com/avatars/{}/{}.png", id, avatar);
    println!("Served request for {}: {}#{}", id, json["username"], json["discriminator"]);
    Ok(avatar_url)
}


async fn resp(arc: Arc<Ctx>, req: Request<Body>) -> anyhow::Result<Response<Body>> {
    fn make_err(err: u16, text: &str) -> anyhow::Result<Response<Body>> {
        return Ok(Response::builder()
            .status(err)
            .body(format!("{} {}", err, text).into())?);
    }

    let x = req.uri().path();
    let request = match x.strip_prefix("/avatar/") {
        None => return make_err(404, "Not found"),
        Some(request) => request,
    };
    let num_id = match request.parse::<u64>() {
        Err(_) => return make_err(404, "Not found"),
        Ok(num) => num,
    };
    let avatar_url = match get_avatar_url(&arc.client, &arc.token, num_id).await {
        Err(_) => return make_err(502, "Discord failed to respond"),
        Ok(avatar_url) => avatar_url,
    };
    let resp = match arc.client.get(avatar_url.parse()?).await {
        Err(_) => return make_err(502, &format!("Discord failed to supply avatar for url: {}", avatar_url)),
        Ok(avatar_data) => avatar_data,
    };
    Ok(Response::builder()
        .status(200)
        .header("content-type", "image/png")
        .body(resp.into_body())?)
}

struct Ctx {
    client: Client<HttpsConnector<HttpConnector>>,
    token: String,
}

async fn wrap_error(arc: Arc<Ctx>, req: Request<Body>) -> anyhow::Result<Response<Body>> {
    return match resp(arc, req).await {
        Err(e) => {
            eprintln!("{:?}", e);
            Ok(Response::builder()
                .status(500)
                .body("500 Internal Error".into())?)
        }
        Ok(o) => Ok(o)
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("TOKEN")?;
    println!("Running with token: {}", token);
    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, Body>(https);
    let arc = Arc::new(Ctx { client, token });
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let service = make_service_fn(|_conn| {
        let carc = Arc::clone(&arc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| { wrap_error(Arc::clone(&carc), req) }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    server.await?;
    Ok(())
}
