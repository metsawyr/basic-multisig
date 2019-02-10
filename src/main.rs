extern crate exonum;
#[macro_use]
extern crate exonum_derive;
use exonum::node::Node;
use exonum::storage::MemoryDB;

mod api;
mod node;
mod proto;
mod schema;
mod service;
mod transaction;
mod wallet;

fn main() {
    exonum::helpers::init_logger().unwrap();

    let node = Node::new(
        MemoryDB::new(),
        vec![Box::new(service::Service)],
        node::get_node_config(),
        None,
    );

    node.run().unwrap();
}
