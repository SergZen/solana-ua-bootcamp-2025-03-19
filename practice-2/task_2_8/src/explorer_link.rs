#[derive(Clone, Debug, PartialEq)]
pub enum Cluster {
    MainnetBeta,
    Testnet,
    Devnet,
    Localnet,
    Custom(String),
}

impl Cluster {
    pub fn as_str(&self) -> String {
        match self {
            Cluster::MainnetBeta => "mainnet-beta".to_string(),
            Cluster::Testnet => "testnet".to_string(),
            Cluster::Devnet => "devnet".to_string(),
            Cluster::Localnet => "localnet".to_string(),
            Cluster::Custom(name) => name.clone(),
        }
    }
}

impl From<&str> for Cluster {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mainnet-beta" => Cluster::MainnetBeta,
            "testnet" => Cluster::Testnet,
            "devnet" => Cluster::Devnet,
            "localnet" => Cluster::Localnet,
            custom => Cluster::Custom(custom.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LinkType {
    Transaction,
    Address,
    Block,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkType::Transaction => "tx",
            LinkType::Address => "address",
            LinkType::Block => "block",
        }
    }
}

impl From<&str> for LinkType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "transaction" | "tx" => LinkType::Transaction,
            "address" => LinkType::Address,
            "block" => LinkType::Block,
            _ => panic!("Invalid link type: {}", s),
        }
    }
}

pub fn get_explorer_link(link_type: &str, id: String, cluster: &str) -> String {
    let link_type = LinkType::from(link_type);
    let cluster = Cluster::from(cluster);
    let base_url = "https://explorer.solana.com";
    let cluster_param = match cluster {
        Cluster::MainnetBeta => "".to_string(),
        _ => format!("?cluster={}", cluster.as_str())
    };
    format!("{}/{}/{}{}", base_url, link_type.as_str(), id, cluster_param)
}