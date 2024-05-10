use std::net::SocketAddr;
use consensus_service::consensus_api_server::{ConsensusApiServer, ConsensusApi};
use consensus_service::{ConsensusRequest, ConsensusResponse};
use tokio_stream::{StreamExt, Stream};
use tonic::{Request, Response, Status, Streaming};
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::{Arc, Mutex};

pub mod consensus_service {
    tonic::include_proto!("consensus_service");
}

#[derive(Default)]
struct ScalarConsensusApi{
    id_map: Arc<Mutex<HashMap<String, i64>>>,
}

#[tonic::async_trait]
impl ConsensusApi for ScalarConsensusApi {
    /// Server streaming response type for the start_stream method.
    // type start_streamStream = Streaming<ConsensusResponse>;

    type start_streamStream = Pin<Box<dyn Stream<Item = Result<ConsensusResponse, Status>> + Send  + 'static>>;
    
    async fn start_stream(
        &self,
        request: Request<Streaming<ConsensusRequest>>,
    ) -> Result<Response<Self::start_streamStream>, Status> 
    {
        let mut request_stream = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(16);

        let response_stream = async_stream::stream! {
            while let Some(tx_hash) = rx.recv().await {
                let response = ConsensusResponse { tx_hash: tx_hash};
                yield Ok(response);
            }
        };

        let id_map = self.id_map.clone();
        tokio::spawn(async move {
            while let Some(req) = request_stream.next().await {
                // Process the request, generate a response
                info!("Receive request {:?}", req);
                let req = req.expect("No data");
                let tx_clone = tx.clone();
                let id_map_clone = id_map.clone();
                tokio::spawn(async move {
                    let _size = id_map_clone.lock().unwrap().len();
                    let tx_hash = *id_map_clone.lock().unwrap().entry(req.tx).or_insert(_size as i64);
                    tx_clone.send(tx_hash).await.expect("Cannot send tx_hash");
                });
            }
        });
        
        Ok(Response::new(Box::pin(response_stream) as Self::start_streamStream))
    }
}

struct ConsensusServer {
    addr: SocketAddr,
    id_map: Arc<Mutex<HashMap<String, i64>>>,
}

impl ConsensusServer {
    fn new(addr: String) -> Self {
        let addr = addr.parse().expect("Wrong address format!!!");
        Self { addr, id_map: Arc::new(Mutex::new(HashMap::new())) }
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Run server");
        let my_scalar_consensus_api = ScalarConsensusApi {id_map: self.id_map.clone()};
        tonic::transport::Server::builder()
            .add_service(ConsensusApiServer::new(my_scalar_consensus_api))
            .serve(self.addr.clone())
            .await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    // Set the created subscriber as the default for the application
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");


    info!("Begin demo");
    let my_consensus_server = ConsensusServer::new("0.0.0.0:3000".to_string());
    my_consensus_server.run().await?;
    Ok(())
}