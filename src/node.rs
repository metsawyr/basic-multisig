use exonum::blockchain::{config::ValidatorKeys, GenesisConfig};
use exonum::node::{NodeApiConfig, NodeConfig};

pub fn get_node_config() -> NodeConfig {
    let (consensus_public_key, consensus_secret_key) = exonum::crypto::gen_keypair();
    let (service_public_key, service_secret_key) = exonum::crypto::gen_keypair();

    let validator_keys = ValidatorKeys {
        consensus_key: consensus_public_key,
        service_key: service_public_key,
    };

    let genesis = GenesisConfig::new(vec![validator_keys].into_iter());

    let api_address = "0.0.0.0:8000".parse().unwrap();

    let api_cfg = NodeApiConfig {
        public_api_address: Some(api_address),
        ..Default::default()
    };

    NodeConfig {
        listen_address: "0.0.0.0:2000".parse().unwrap(),
        external_address: "0.0.0.0:2000".parse().unwrap(),
        service_public_key,
        service_secret_key,
        consensus_public_key,
        consensus_secret_key,
        genesis,
        connect_list: Default::default(),
        database: Default::default(),
        network: Default::default(),
        api: api_cfg,
        thread_pool_size: Default::default(),
        mempool: Default::default(),
        services_configs: Default::default(),
    }
}
