use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use hello::ThreadPool;

fn main() {
    //监听7878端口
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    //创建线程池
    let pool = ThreadPool::new(4);

    //读取请求流
    for stream in listener.incoming().take(2) {
        let s = stream.unwrap();

        pool.execute(|| {
            handle_connection(s);
        })
    }
}

/// 处理请求
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    //每次读1024字节
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    //匹配请求
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    //读取页面内容
    let contents = fs::read_to_string(filename).unwrap();

    //响应信息
    let response = format!("{}{}", status_line, contents);
    //写入
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
