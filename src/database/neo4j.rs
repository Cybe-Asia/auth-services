use neo4rs::Graph;

pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Graph, neo4rs::Error> {
    Graph::new(uri, user, password).await
}
