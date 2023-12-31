use tokio_postgres::{Client, NoTls, tls::NoTlsStream};

pub async fn connect_pg() -> (Client, tokio_postgres::Connection<tokio_postgres::Socket, NoTlsStream>) {
    tokio_postgres::connect("host=195.201.60.244 user=sparkcloud password=Nn2Zyn3PaF2jK9eLMt8V dbname=SparkCloud", NoTls).await.unwrap()
}
