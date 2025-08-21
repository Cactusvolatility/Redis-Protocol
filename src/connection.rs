use bytes::{BytesMut, Buf};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use mini_redis::{Frame,Result};
use mini_redis::frame::Error::Incomplete;
use std::io::Cursor;


pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // allocate with 4KB of capacity
            buffer: vec![0;4096],
            cursor: 0,
        }
    }
}

pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
    loop {
        // use if let to check
            // if error then ignore
            // otherwise unwrap the value with Ok(Some())
        if let Some(frame) = self.parse_frame()? {
            return Ok(Some(frame));
        }

        // heap allocation is increased - what happens is it gets doubled then old data is copied over
            // vec remains one contiguous heap block
            // overhead is temporary copy of buffer and temporary extra memory
        if self.buffer.len() == self.cursor {
            self.buffer.resize(self.cursor * 2, 0);
        }

        // if the buffer is empty then what?
            // Case 1: clean close - buffer is empty
            // Case 2: dirty close - buffer is non-empty but I didn't receive anything with read_buf
        // manual rewrite:

        let n = self.stream.read(
            // this is from cursor to end (the unfilled portion)
            &mut self.buffer[self.cursor..]).await?;

        
        if 0 == n {
            if self.cursor == 0 {
                return Ok(None);
            } else {
                return Err("connection reser by peer - buffer is nonempty".into());
            }
        } else {
            self.cursor += n;
        }
    }
}

fn parse_frame(&mut self) -> Result<Option<Frame>> {
    // idiomatic?
        // can use .buffer[..] or just .buffer
    let mut buf = Cursor::new(&self.buffer[..]);

    match Frame::check(&mut buf) {
        Ok (_) => {
            // get the byte length of the frame
                // what is .position?
                    // Cursor has a seek trait >> returns the offset from the start of the buffer
            let len = buf.position() as usize;

            // set to 0 to call parse
            buf.set_position(0);
            
            // parse the frame ( the buffer )
            let frame = Frame::parse(&mut buf);

            // then go back to where we left off
            self.buffer.advance(len);

            // return the part that we parsed
            Ok(Some(frame))
        }

        // 2 case
            // Case 1: not enough data was buffered
            // or encountered another error 
        Err(Incomplete) => Ok(None), // return None, it's fine
        Err(e) => Err(e.into()), // convert to err type? I guess
    }
}