use std::net::SocketAddr;
use consensus_service::consensus_api_server::{ConsensusApiServer, ConsensusApi};
use consensus_service::{ConsensusRequest, ConsensusResponse};
use tokio_stream::{StreamExt, Stream};
use tonic::{Request, Response, Status, Streaming};
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub mod consensus_service {
    tonic::include_proto!("consensus_service");
}

#[derive(Default)]
struct ScalarConsensusApi;

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
        let mut id_map: HashMap<String, i64> = HashMap::new();

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        while let Some(req) = request_stream.next().await {
            // Process the request, generate a response
            let tx_clone = tx.clone();
            info!("Receive request {:?}", req);
            let req = req.expect("No data");
            let _size = id_map.len();
            let tx_hash = *id_map.entry(req.tx)
                .or_insert(_size as i64);
            tokio::spawn(async move {
                tx_clone.send(tx_hash).await.expect("Cannot send tx");
            });
            
        }

        let response_stream = async_stream::stream! {
            while let Some(tx_hash) = rx.recv().await {
                let response = ConsensusResponse { tx_hash: tx_hash};
                yield Ok(response);
            }
        };
        
        Ok(Response::new(Box::pin(response_stream) as Self::start_streamStream))
    }
}

struct ConsensusServer {
    addr: SocketAddr,
}

impl ConsensusServer {
    fn new(addr: String) -> Self {
        let addr = addr.parse().expect("Wrong address format!!!");
        Self { addr }
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Run server");
        let my_scalar_consensus_api = ScalarConsensusApi::default();
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