use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct CliConfig {
    #[envconfig(from = "DYNDNSD_HETZNER_API_TOKEN")]
    pub api_token: String,
    #[envconfig(from = "DYNDNSD_DOMAIN")]
    pub domain: String,
    #[envconfig(from = "DYNDNSD_SUBDOMAIN")]
    pub subdomain: String,
    #[envconfig(from = "DYNDNSD_WAN_INTERFACE")]
    pub interface: String,
    #[envconfig(from = "DYNDNSD_POLL_INTERVAL")]
    pub interval: u32,
}
