use tokio::net::{TcpListener, TcpStream};
use mini_redis::{cmd, frame, Connection, Frame};

// shared state
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; 
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
        // just assigning a TCP listenner
        // await due to async
            // what am I unwrapping - TcpListener from the bind function
    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {

        // await means that it'll wait while other threads run - non-blocking since it yeilds control over
            // this is basically context_switch

        let (socket, _) = listener.accept().await.unwrap();

        // we increment Arc counter
            // cloning the handle to the hashmap
        let db = db.clone();

        println!("Accepted");

        // we need this to be concurrent for many requests
                    // so better resource utilization
                    // 1 task = 1 handle
        tokio::spawn(async move {
            process(socket,db).await;
        });
    }
}

async fn process(socket:TcpStream, db: Db) {
    // reminder:
        /* This is equivalent to:
            use mini_redis::Command;        // The enum type itself
            use mini_redis::Command::Get;   // The Get variant
            use mini_redis::Command::Set;   // The Set variant */
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // we use a Hashmap to store values from incoming commands
    //let mut db = HashMap::new(); -- moved to main()

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
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
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