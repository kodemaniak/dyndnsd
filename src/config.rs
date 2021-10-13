use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct CliConfig {
    #[envconfig(from = "DYNDNSD_HETZNER_API_TOKEN")]
    pub api_token: String,
    #[envconfig(from = "DYNDNSD_DOMAIN")]
    pub domain: String,
    #[envconfig(from = "DYNDNSD_SUBDOMAIN")]
    pub subdomain: String,
    #[envconfig(from = "DYNDNSD_POLL_INTERVAL")]
    pub interval: u32,
    #[envconfig(from = "DYNDNSD_UBUS_URL")]
    pub ubus_url: String,
    #[envconfig(from = "DYNDNSD_UBUS_USER")]
    pub ubus_user: String,
    #[envconfig(from = "DYNDNSD_UBUS_SECRET")]
    pub ubus_secret: String,
}
