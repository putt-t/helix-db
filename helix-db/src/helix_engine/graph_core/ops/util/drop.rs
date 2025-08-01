use crate::helix_engine::{
    bm25::bm25::BM25,
    graph_core::ops::tr_val::TraversalVal,
    storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods},
    types::GraphError,
};
use heed3::RwTxn;
use std::{fmt::Debug, sync::Arc};

pub struct Drop<I> {
    pub iter: I,
}

impl<T> Drop<Vec<Result<T, GraphError>>>
where
    T: IntoIterator<Item = TraversalVal> + Debug,
{
    pub fn drop_traversal(
        iter: T,
        storage: Arc<HelixGraphStorage>,
        txn: &mut RwTxn,
    ) -> Result<(), GraphError> {
        iter.into_iter()
            .try_for_each(|item| -> Result<(), GraphError> {
                match item {
                    TraversalVal::Node(node) => match storage.drop_node(txn, &node.id) {
                        Ok(_) => {
                            if let Some(bm25) = &storage.bm25 {
                                if let Err(e) = bm25.delete_doc(txn, node.id) {
                                    println!("failed to delete doc from bm25: {e}");
                                }
                            }
                            println!("Dropped node: {:?}", node.id);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                    TraversalVal::Edge(edge) => match storage.drop_edge(txn, &edge.id) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    },
                    TraversalVal::Vector(vector) => match storage.drop_vector(txn, &vector.id) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    },
                    _ => Err(GraphError::ConversionError(format!(
                        "Incorrect Type: {item:?}"
                    ))),
                }
            })
    }
}
