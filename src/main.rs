use clokwerk::{Scheduler, TimeUnits};
use dyndnsd::{
    config::CliConfig, dns_service::HetznerDnsService, dyndns_service::DynDnsService,
    ubus_jsonrpc_public_ip_service::UbusJsonRpcPublicIpService,
};
use envconfig::Envconfig;
use std::time::Duration;
use tokio::sync::mpsc::channel;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()> {
    let config = CliConfig::init_from_env().unwrap();

    let mut scheduler = Scheduler::new();

    let dns_service = HetznerDnsService::new(&config.api_token);
    let netlink =
        UbusJsonRpcPublicIpService::new(&config.ubus_url, &config.ubus_user, &config.ubus_secret);

    let dyndns = DynDnsService::new(
        &config.domain,
        &config.subdomain,
        Box::new(dns_service),
        Box::new(netlink),
    );

    let (tx, mut rx) = channel::<()>(1);

    let loop_tx = tx.clone();
    scheduler.every(config.interval.seconds()).run(move || {
        loop_tx.clone().blocking_send(()).unwrap();
    });
    let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    tx.clone().send(()).await.unwrap();

    while let Some(_) = rx.recv().await {
        dyndns.update_dns_if_required().await.map_err(|_| ())?;
    }

    return Err(());
}
