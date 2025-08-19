use tokio::net::{TcpListener, TcpStream};
use mini_redis::{cmd, frame, Connection, Frame};

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; 

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
        // just assigning a TCP listenner
        // await due to async
            // what am I unwrapping - TcpListener from the bind function
    loop {

        // await means that it'll wait while other threads run - non-blocking since it yeilds control over

            // this is basically context_switch
            
            // but it's not the same as leaving it out to be blocking since it needs to unwrap Future<T>
                // basically use async w/ I/O work

                    // SO!!!!
                        // await unwraps Future<T>
                        // then .unwrap() to unwrap Result<T,E>

        let (socket, _) = listener.accept().await.unwrap();

        // we need this to be concurrent for many requests

            // this is a Tokio task
                // function.await is sequential
                // tasks are concurrent (not parallel)

                    // so better resource utilization
                 
                    // 1 task = 1 handle

        tokio::spawn(async move {

            process(socket).await;
        });
    }
}

async fn process(socket:TcpStream) {
    // reminder:
        /* This is equivalent to:
            use mini_redis::Command;        // The enum type itself
            use mini_redis::Command::Get;   // The Get variant
            use mini_redis::Command::Set;   // The Set variant */
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // we use a Hashmap to store values from incoming commands
    let mut db = HashMap::new();

    // connection lets use read/write redis frames aparently
    let mut connection = Connection::new(socket);

    // as long as the frame exists - we will read from it

        // this is the Redis Protocol
    while let Some(frame) = connection.read_frame().await.unwrap() {
        // Command enum has 2 types of
            // Get
            // Set

                // then we insert into our hashmap
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        // we don't need to give it un-necessary ownership - use &response
        connection.write_frame(&response).await.unwrap()
    }
    /* 
    if let Some(frame) = connection.read_frame().await.unwrap() 
    {
        // print out the enum Frame
            /*
                Simple( /* … */ ),
                Error( /* … */ ),
                Integer( /* … */ ),
            */
        println!("GOT: {:?}", frame);

        // just leave it with unimplemented for now
        let response = Frame::Error("uninmplemented".to_string());
        // write a frame to the stream
            // stream is the network connection
            // writing is sending bytes over it

                // Serialize - convert Frame Struct
                // Buffer - store bytes in memory buffer
                    // !Note - this is talking about Application Buffer, not OS buffer
                // Flush - send entire buffer to network
                // Transmit - bytes travel over network
        connection.write_frame(&response).await.unwrap();
    }
    */
}