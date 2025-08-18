mod request;
mod response;

use clap::Parser;
use rand::{Rng, SeedableRng};
use tokio::{net::TcpListener, net::TcpStream};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;
use std::net::{IpAddr};

// use threadpool::ThreadPool;
// use std::thread;

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[command(about = "Fun with load balancing")]
struct CmdOptions {
    /// "IP/port to bind to"
    #[arg(short, long, default_value = "0.0.0.0:1100")]
    bind: String,
    /// "Upstream host to forward requests to"
    #[arg(short, long)]
    upstream: Vec<String>,
    /// "Perform active health checks on this interval (in seconds)"
    #[arg(long, default_value = "10")]
    active_health_check_interval: usize,
    /// "Path to send request to for active health checks"
    #[arg(long, default_value = "/")]
    active_health_check_path: String,
    /// "Maximum number of requests to accept per IP per minute (0 = unlimited)"
    #[arg(long, default_value = "0")]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
// #[derive(Clone)]
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks (Milestone 4)
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    max_requests_per_minute: usize,
    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,
    alive_upstream_status: RwLock<(usize, Vec<bool>)>,
    rate_limit_counter: Mutex<HashMap<IpAddr, usize>>

}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // let num_threads = 8;
    // let pool = ThreadPool::new(num_threads);

    // let mut threads = Vec::new();

    // Start listening for connections
    let mut listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    let total_upstream = options.upstream.len();
    // Handle incoming connections
    let state = ProxyState {
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        alive_upstream_status: RwLock::new((total_upstream, vec![true; total_upstream])),
        rate_limit_counter: Mutex::new(HashMap::new())
    };

    // for stream in listener.incoming() {
    //     if let Ok(stream) = stream {
    //         // Handle the connection!
    //         // No thread
    //         // handle_connection(stream, &state);

    //         // Thread for Each Connection
    //         // let stream_thread = stream.try_clone();
    //         // let state_thread = state.clone();
    //         // threads.push(thread::spawn(move || {
    //         //     handle_connection(stream_thread.unwrap(), &state_thread);
    //         // }))

    //         // Thread pool
    //         // let stream_thread = stream.try_clone();
    //         // let state_thread = state.clone();
    //         // pool.execute(move|| {
    //         //     handle_connection(stream_thread.unwrap(), &state_thread);
    //         // });
    //     }
    // }
    let shared_state = Arc::new(state);

    if shared_state.max_requests_per_minute > 0 {
        let state_rate_ref = shared_state.clone();
        tokio::spawn(async move {
            rate_limit_count_refresher(state_rate_ref, 60).await;
        });
    }

    let state_health_ref = shared_state.clone();
    tokio::spawn(async move {
        active_health_check(state_health_ref).await;
    });


    loop {
        match listener.accept().await {
            Ok((mut stream, socket)) => {
                if shared_state.max_requests_per_minute > 0 {
                    let mut rate_limit_counter = shared_state.rate_limit_counter.lock().unwrap();
                    let ip_addr = socket.ip();
                    let count = rate_limit_counter.entry(ip_addr).or_insert(0);
                    log::debug!("addr: {}, count: {}", ip_addr, count);
                    *count += 1;
                    if *count > shared_state.max_requests_per_minute {
                        let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
                        response::write_to_stream(&response, &mut stream).await.unwrap();
                        continue;
                    }               
                }
                let shared_state_ref = shared_state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, shared_state_ref).await;
                });
            },
            Err(_) => {
                break;
            },
        }

    }
}

async fn rate_limit_count_refresher(state: Arc<ProxyState>, interval: u64) {
    sleep(Duration::from_secs(interval)).await;
    let mut rate_limit_counter = state.rate_limit_counter.lock().unwrap();
    rate_limit_counter.clear();
}

async fn pick_random_alive_server(state: &Arc<ProxyState>) -> Result<usize, Error> {
    let upstream_servers_status = state.alive_upstream_status.read().await;
    if upstream_servers_status.0 == 0 {
        return Err(Error::new(ErrorKind::Other, "All Upstream Servers are down!"));
    }

    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut upstream_idx;

    loop {
        upstream_idx = rng.gen_range(0..state.upstream_addresses.len());
        if upstream_servers_status.1[upstream_idx] {
            return Ok(upstream_idx);
        }
    }
}

async fn connect_to_server(state: &Arc<ProxyState>, upstream_idx: usize) -> Result<TcpStream, Error> {
    let upstream_ip = &state.upstream_addresses[upstream_idx];
    match TcpStream::connect(upstream_ip).await {
        Ok(stream) => return Ok(stream),
        Err(err) => {
            log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
            return Err(err)
        }
    }
}

async fn check_server(state: &Arc<ProxyState>, idx: usize, path: &String) -> Option<usize> {
    let mut stream  = connect_to_server(state, idx).await.ok()?;
    let upstream_ip = &state.upstream_addresses[idx];
    let request = http::Request::builder()
    .method(http::Method::GET)
    .uri(path)
    .header("Host", upstream_ip)
    .body(Vec::new())
    .unwrap();
    let _ = request::write_to_stream(&request, &mut stream).await.ok()?;
    let res = response::read_from_stream(&mut stream, &http::Method::GET).await.ok()?;
    if res.status().as_u16() != 200 {
        return None;
    } else {
        return Some(1);
    }
}

async fn active_health_check(state: Arc<ProxyState>) {
    let interval = state.active_health_check_interval as u64;
    let path = &state.active_health_check_path;

    loop {
        sleep(Duration::from_secs(interval)).await;
        let mut upstream_status_mut = state.alive_upstream_status.write().await;
        for idx in 0..(upstream_status_mut.1.len()) {
            if check_server(&state, idx, path).await.is_some() {
                if !upstream_status_mut.1[idx] {
                    upstream_status_mut.0 += 1;
                    upstream_status_mut.1[idx] = true;
                }
            } else {
                if upstream_status_mut.1[idx] {
                    upstream_status_mut.0 -= 1;
                    upstream_status_mut.1[idx] = false;
                }
            }
        }
    }
}

async fn connect_to_upstream(state: Arc<ProxyState>) -> Result<TcpStream, std::io::Error> {
    loop {
        match pick_random_alive_server(&state).await {
            Ok(upstream_idx) => {
                match connect_to_server(&state, upstream_idx).await {
                    Ok(stream) => return Ok(stream),
                    Err(_) => {
                        let mut upstream_servers_status_mut = state.alive_upstream_status.write().await;
                        upstream_servers_status_mut.0 -= 1;
                        upstream_servers_status_mut.1[upstream_idx] = false;
                    }
                }
            },
            Err(err) => {
                log::error!("Cannot find a server. Error: {}", err);
                return Err(err);
            }
        } 
    }
    // TODO: implement failover (milestone 3)
}



async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!(
        "{} <- {}",
        client_ip,
        response::format_response_line(&response)
    );
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn handle_connection(mut client_conn: TcpStream, state: Arc<ProxyState>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);
    
    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;

        }
    };
    let upstream_ip = upstream_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!(
                "Failed to send request to upstream {}: {}",
                upstream_ip,
                error
            );
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
