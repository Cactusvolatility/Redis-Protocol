use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

enum MainFuture {
    State0,
    State1(Delay),
    Terminated,
}

impl Future for MainFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        use MainFuture::*;

        loop {
            match *self {
                // init State 0
                State0 => {
                    let when = Instant::now() + Dureation::from_millis(10);
                    // Delay has its own poll representation that returns done when finished 
                    let future = Delay { when };
                    *self = State1(future);
                }
                // State 1
                    // we use ref mut because we don't want to move ownership out of the enum
                State1(ref mut my_future) => {
                    // our own poll implementation
                        // we need to wrap my_future for our own .poll(cx) to take (see line 15)
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }

                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}