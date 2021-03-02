use hyper::service::Service;
use core::task::{Context, Poll};
use core::future::Future;
use std::pin::Pin;
use std::io::Cursor;
use hyper::client::connect::{Connection, Connected};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Clone)]
pub struct CustomTransporter;

unsafe impl Send for CustomTransporter {}

impl CustomTransporter {
    pub fn new() -> CustomTransporter {
        CustomTransporter{}
    }
}

impl Connection for CustomTransporter {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

pub struct CustomResponse {
    w: Cursor<Vec<u8>>,
    i: i32
}

unsafe impl Send for CustomResponse {
    
}

impl Connection for CustomResponse {
    fn connected(&self) -> Connected {
        println!("connected");
        Connected::new()
    }
}

impl AsyncRead for CustomResponse {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>
    ) -> Poll<std::io::Result<()>> {
        self.i+=1;
        if self.i >=3 {
            println!("!!!!!!!!!!!!poll_read for buf size {}", buf.capacity());
            let r = Pin::new(&mut self.w).poll_read(cx, buf);
            println!("did poll_read");
            r
        } else {
            println!("poll read pending, i={}", self.i);
            Poll::Pending
        }
    }
}

impl AsyncWrite for CustomResponse {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]
    ) -> Poll<Result<usize, std::io::Error>>{
        //let v = vec!();
        println!("poll_write____");

        let r= Pin::new(&mut self.w).poll_write(cx, buf);

        let s = match std::str::from_utf8(buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        println!("result: {}, size: {}, i: {}", s, s.len(), self.i);
        if self.i>=0{
            r
            //Poll::Ready(Ok(s.len()))
        }else{
            println!("poll_write pending");
            Poll::Pending
        }
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), std::io::Error>> {
        println!("poll_flush");
        let r = Pin::new(&mut self.w).poll_flush(cx);
        if self.i>=0{
            println!("DID poll_flush");
            r
            //Poll::Ready(Ok(()))
        }else{
            println!("poll_flush pending");
            Poll::Pending
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<(), std::io::Error>>
    {
        println!("poll_shutdown");
        Pin::new(&mut self.w).poll_shutdown(cx)
    }
}


impl Service<hyper::Uri> for CustomTransporter {
    type Response = CustomResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        println!("poll_ready");
        Poll::Ready(Ok(()))
        //Poll::Pending
    }

    fn call(&mut self, req: hyper::Uri) -> Self::Future {
        println!("call");
        // create the body
        let body: Vec<u8> = "HTTP/1.1 200 OK\nDate: Mon, 27 Jul 2009 12:28:53 GMT\nServer: Apache/2.2.14 (Win32)\nLast-Modified: Wed, 22 Jul 2009 19:15:56 GMT\nContent-Length: 88\nContent-Type: text/html\nConnection: Closed<html><body><h1>Hello, World!</h1></body></html>".as_bytes()
            .to_owned();
        // Create the HTTP response
        let resp = CustomResponse{
            w: Cursor::new(body),
            i: 0
        };
         
        // create a response in a future.
        let fut = async {
            Ok(resp)
        };
        println!("gonna return from call");
        // Return the response as an immediate future
        Box::pin(fut)
    }
}