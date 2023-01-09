use std::ops::{Deref, DerefMut};

use petgraph::adj::NodeIndex;
use petgraph::dot::{Config, Dot};
use petgraph::{Directed, Graph};

use crate::agent::AgentKind;
use crate::util::{BaseInt, BaseUint, ModelError};

/// A newtype struct for encapsulating the graph. It defines the types of the
/// graph.
#[derive(Default)]
pub struct MyGraph {
    /// Inner graph
    pub content: Graph<BaseInt, BaseInt, Directed>,
}

impl MyGraph {
    /// Get children from graph using the graph index of the parent.
    pub fn get_children(&self, node_index: &BaseInt) -> Result<Vec<BaseInt>, ModelError> {
        let mut res: Vec<BaseInt> = vec![];
        let node = NodeIndex::from(*node_index as BaseUint);
        for neighbour in self.neighbors(node) {
            res.push(neighbour.index() as BaseInt);
        }
        res.sort_unstable();
        Ok(res)
    }

    /// Provide the dotstring from the inner graph.
    pub fn get_dot_string(&self) -> String { Dot::with_config(&self.content, &[Config::EdgeNoLabel]).to_string() }

    /// Directed edge add
    pub fn add_edge(&mut self, from: BaseUint, to: BaseUint) -> Result<(), ModelError> {
        self.content
            .add_edge(NodeIndex::from(from as BaseUint), NodeIndex::from(to as BaseUint), 1);
        Ok(())
    }

    /// Add node. Return index `BaseUint`
    pub fn add_node(&mut self, agent_kind: AgentKind, agent_index: BaseUint) -> Result<(), ModelError> {
        let graph_index = self.content.add_node(agent_index as i32).index() as BaseUint;
        if graph_index == agent_index {
            Ok(())
        } else {
            Err(ModelError::GraphError(format!(
                "Something went wrong with creating a {agent_kind:?} node with index {agent_index}"
            )))
        }
    }
}

impl DerefMut for MyGraph {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.content }
}
impl Deref for MyGraph {
    type Target = Graph<BaseInt, BaseInt, Directed>;

    fn deref(&self) -> &Self::Target { &self.content }
}
