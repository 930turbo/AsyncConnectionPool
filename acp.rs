use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::sync::Arc;
use tokio::sync::Mutex;

// Define a struct for the connection pool that holds a vector of connections, 
// a receiver for release requests and a sender for connection requests
struct ConnectionPool {
    conns: Arc<Mutex<Vec<Connection>>>, // wrapped with Arc and Mutex
    rx: Arc<Mutex<mpsc::Receiver<oneshot::Sender<Connection>>>>, // wrapped with Arc and Mutex
    tx: Arc<Mutex<mpsc::Sender<oneshot::Receiver<Connection>>>>, // wrapped with Arc and Mutex
}

impl ConnectionPool {
    // Create a new connection pool with a given size
    async fn new(size: usize) -> Self {
        // Create the channel for sending and receiving release requests and connection requests
        let (tx, rx) = mpsc::channel(size);
        // Create a vector of connections with the given size
        let conns = Arc::new(Mutex::new((0..size).map(|_| Connection::new()).collect()));
        let rx = Arc::new(Mutex::new(rx));
        let tx = Arc::new(Mutex::new(tx));
        // Return the connection pool
        Self { conns, rx, tx }
    }

    // Get a connection from the pool
    async fn get_conn(&self) -> Connection {
        // Create a channel for sending and receiving a connection
        let (tx, rx) = oneshot::channel();
        // Send the request for a connection through the sender
        self.tx.lock().await.send(tx).await.unwrap();
        // Wait for the connection to be received
        rx.await.unwrap()
    }

    // Release a connection back to the pool
    async fn release_conn(&self, conn: Connection) {
        // Wait for a release request
        let tx = self.rx.lock().await.recv().await.unwrap();
        // Send the connection back through the request channel
        tx.send(conn).unwrap();
    }
}
