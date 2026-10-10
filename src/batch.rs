use crate::client::GameJoltClient;
use crate::error::Result;
use serde_json::Value;

pub struct BatchBuilder<'a> {
    client: &'a GameJoltClient,
    operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug)]
pub struct BatchOperation {
    pub endpoint: String,
    pub parameters: Vec<(String, String)>,
}

impl<'a> BatchBuilder<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self {
            client,
            operations: Vec::new(),
        }
    }

    pub fn add(mut self, operation: BatchOperation) -> Self {
        self.operations.push(operation);
        self
    }

    pub async fn execute(self) -> Result<Vec<Value>> {
        let mut results = Vec::with_capacity(self.operations.len());
        for chunk in self.operations.chunks(50) {
            for operation in chunk {
                results.push(
                    self.client
                        .request::<Value>(&operation.endpoint, &operation.parameters)
                        .await?,
                );
            }
        }
        Ok(results)
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl BatchOperation {
    pub fn new(endpoint: impl Into<String>, parameters: Vec<(String, String)>) -> Self {
        Self {
            endpoint: endpoint.into(),
            parameters,
        }
    }
}
