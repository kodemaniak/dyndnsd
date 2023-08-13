use clokwerk::{Scheduler, TimeUnits};
use dyndnsd::{
    config::CliConfig,
    dns_service::HetznerDnsService,
    dyndns_service::{DynDnsService, DynDnsServiceError},
    ubus_jsonrpc_public_ip_service::UbusJsonRpcClient,
};
use envconfig::Envconfig;
use std::time::Duration;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), DynDnsServiceError> {
    env_logger::init();

    let config = CliConfig::init_from_env().unwrap();

    let mut scheduler = Scheduler::new();

    let dns_service = HetznerDnsService::new(&config.api_token);
    let ubus_service =
        UbusJsonRpcClient::new(&config.ubus_url, &config.ubus_user, &config.ubus_secret);

    let dyndns = DynDnsService::new(
        &config.domain,
        &config.subdomain,
        Box::new(dns_service),
        Box::new(ubus_service),
    );

    let (tx, mut rx) = channel::<()>(1);

    let loop_tx = tx.clone();
    scheduler.every(config.interval.seconds()).run(move || {
        loop_tx.clone().blocking_send(()).unwrap();
    });
    let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    tx.clone().send(()).await.unwrap();

    while rx.recv().await.is_some() {
        dyndns.update_dns_if_required().await?;
    }

    Ok(())
}
