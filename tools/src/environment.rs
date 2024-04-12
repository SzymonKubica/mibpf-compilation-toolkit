use std::{env, path::Path};

pub struct Environment {
    pub mibpf_root_dir: String,
    pub coap_root_dir: String,
    pub riot_instance_net_if: String,
    pub riot_instance_ip: String,
    pub host_net_if: String,
    pub host_ip: String,
    pub board_name: String,
}

pub fn load_env() -> Environment {
    let path_str = env::var("DOTENV").unwrap_or_else(|_| ".env".to_string());
    let path = Path::new(&path_str);
    let _ = dotenv::from_path(path);

    Environment {
        mibpf_root_dir: dotenv::var("MIBPF_ROOT_DIR").unwrap_or_else(|_| "..".to_string()),
        coap_root_dir: dotenv::var("COAP_ROOT_DIR").unwrap_or_else(|_| "../coaproot".to_string()),
        riot_instance_net_if: dotenv::var("RIOT_INSTANCE_NET_IF")
            .unwrap_or_else(|_| "6".to_string()),
        riot_instance_ip: dotenv::var("RIOT_INSTANCE_IP")
            .unwrap_or_else(|_| "fe80::a0d9:ebff:fed5:986b".to_string()),
        host_net_if: dotenv::var("HOST_NET_IF").unwrap_or_else(|_| "tapbr0".to_string()),
        host_ip: dotenv::var("HOST_IP").unwrap_or_else(|_| "fe80::cc9a:73ff:fe4a:47f6".to_string()),
        board_name: dotenv::var("BOARD_NAME").unwrap_or_else(|_| "native".to_string()),
    }
}
