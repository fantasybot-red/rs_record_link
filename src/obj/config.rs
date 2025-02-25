

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub bind_addr: String,
    pub auth: Option<String>
}

impl Config {
    pub fn load_config() -> Result<Self, String> {

        let bind_addr_r = std::env::var("BIND_ADDR");
        if bind_addr_r.is_err() {
            return Err("No BIND_ADDR provided".to_string());
        }
        let bind_addr = bind_addr_r.unwrap();

        let auth = std::env::var("AUTH").ok();

        Ok(Self {
            bind_addr,
            auth
        })
    }
}