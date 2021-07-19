use crate::{
    dns_service::{DnsService, DnsServiceError},
    netlink_public_ip_service::{PublicIpService, PublicIpServiceError},
};

pub struct DynDnsService {
    domain: String,
    subdomain: String,
    dns_service: Box<dyn DnsService>,
    public_ip_service: Box<dyn PublicIpService>,
}

impl DynDnsService {
    pub fn new(
        domain: &str,
        subdomain: &str,
        dns_service: Box<dyn DnsService>,
        public_ip_service: Box<dyn PublicIpService>,
    ) -> Self {
        Self {
            domain: String::from(domain),
            subdomain: String::from(subdomain),
            dns_service,
            public_ip_service,
        }
    }

    pub async fn update_dns_if_required(&self) -> Result<(), DynDnsServiceError> {
        let current_dns_ip = self
            .dns_service
            .resolve_ip(&self.subdomain, &self.domain)
            .await?;
        let current_local_ip = self.public_ip_service.get_ip()?;

        if current_dns_ip.is_none() || current_dns_ip.unwrap() != current_local_ip {
            self.dns_service
                .update_dns(&self.subdomain, &self.domain, current_local_ip)
                .await?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum DynDnsServiceError {
    UnknownError,
}

impl From<DnsServiceError> for DynDnsServiceError {
    fn from(_: DnsServiceError) -> Self {
        DynDnsServiceError::UnknownError
    }
}

impl From<PublicIpServiceError> for DynDnsServiceError {
    fn from(_: PublicIpServiceError) -> Self {
        DynDnsServiceError::UnknownError
    }
}

#[cfg(test)]
mod tests {
    use mockall::*;
    use std::net::Ipv4Addr;

    use super::*;
    use crate::dns_service::*;
    use crate::netlink_public_ip_service::*;

    #[tokio::test]
    async fn do_nothing_when_dns_matches_ip() {
        let local_ip = Ipv4Addr::new(127, 0, 0, 2);

        let mut dns_svc_mock = Box::new(MockDnsService::new());
        dns_svc_mock
            .expect_resolve_ip()
            .with(predicate::eq("test"), predicate::eq("example.com"))
            .times(1)
            .returning(move |_, _| Ok(Some(local_ip.clone())));
        dns_svc_mock
            .expect_update_dns()
            .with(
                predicate::eq("test"),
                predicate::eq("example.com"),
                predicate::eq(local_ip),
            )
            .never()
            .returning(|_, _, _| Ok(()));

        let mut netlink_svc_mock = Box::new(MockPublicIpService::new());
        netlink_svc_mock
            .expect_get_ip()
            .times(1)
            .returning(move || Ok(local_ip));

        let kernel = DynDnsService::new("example.com", "test", dns_svc_mock, netlink_svc_mock);
        kernel
            .update_dns_if_required()
            .await
            .expect("calling failed");
    }

    #[tokio::test]
    async fn initialize_when_dns_not_matches_ip() {
        let local_ip = Ipv4Addr::new(127, 0, 0, 2);

        let mut dns_svc_mock = Box::new(MockDnsService::new());
        dns_svc_mock
            .expect_resolve_ip()
            .with(predicate::eq("test"), predicate::eq("example.com"))
            .times(1)
            .returning(|_, _| Ok(Some(Ipv4Addr::new(127, 0, 0, 3))));
        dns_svc_mock
            .expect_update_dns()
            .with(
                predicate::eq("test"),
                predicate::eq("example.com"),
                predicate::eq(local_ip),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let mut netlink_svc_mock = Box::new(MockPublicIpService::new());
        netlink_svc_mock
            .expect_get_ip()
            .times(1)
            .returning(move || Ok(local_ip));

        let kernel = DynDnsService::new("example.com", "test", dns_svc_mock, netlink_svc_mock);
        kernel
            .update_dns_if_required()
            .await
            .expect("calling failed");
    }

    #[tokio::test]
    async fn error_on_initialize_during_dns_service_lookup() {
        let local_ip = Ipv4Addr::new(127, 0, 0, 2);

        let mut dns_svc_mock = Box::new(MockDnsService::new());
        dns_svc_mock
            .expect_resolve_ip()
            .with(predicate::eq("test"), predicate::eq("example.com"))
            .times(1)
            .returning(|_, _| Err(DnsServiceError::UnknownError));
        dns_svc_mock
            .expect_update_dns()
            .with(
                predicate::eq("test"),
                predicate::eq("example.com"),
                predicate::eq(local_ip),
            )
            .never();

        let mut netlink_svc_mock = Box::new(MockPublicIpService::new());
        netlink_svc_mock.expect_get_ip().never();

        let kernel = DynDnsService::new("example.com", "test", dns_svc_mock, netlink_svc_mock);
        assert_eq!(
            Err(DynDnsServiceError::UnknownError),
            kernel.update_dns_if_required().await
        );
    }

    #[tokio::test]
    async fn error_on_initialize_during_rtnetlink_lookup() {
        let local_ip = Ipv4Addr::new(127, 0, 0, 2);

        let mut dns_svc_mock = Box::new(MockDnsService::new());
        dns_svc_mock
            .expect_resolve_ip()
            .with(predicate::eq("test"), predicate::eq("example.com"))
            .times(1)
            .returning(move |_, _| Ok(Some(local_ip.clone())));
        dns_svc_mock
            .expect_update_dns()
            .with(
                predicate::eq("test"),
                predicate::eq("example.com"),
                predicate::eq(local_ip),
            )
            .never();

        let mut netlink_svc_mock = Box::new(MockPublicIpService::new());
        netlink_svc_mock
            .expect_get_ip()
            .times(1)
            .returning(move || Err(PublicIpServiceError::UnknownError));

        let kernel = DynDnsService::new("example.com", "test", dns_svc_mock, netlink_svc_mock);
        assert_eq!(
            Err(DynDnsServiceError::UnknownError),
            kernel.update_dns_if_required().await
        );
    }

    #[tokio::test]
    async fn error_on_initialize_during_dns_update() {
        let local_ip = Ipv4Addr::new(127, 0, 0, 2);
        let remote_ip = Ipv4Addr::new(127, 0, 0, 3);

        let mut dns_svc_mock = Box::new(MockDnsService::new());
        dns_svc_mock
            .expect_resolve_ip()
            .with(predicate::eq("test"), predicate::eq("example.com"))
            .times(1)
            .returning(move |_, _| Ok(Some(remote_ip)));
        dns_svc_mock
            .expect_update_dns()
            .with(
                predicate::eq("test"),
                predicate::eq("example.com"),
                predicate::eq(local_ip),
            )
            .times(1)
            .returning(|_, _, _| Err(DnsServiceError::UnknownError));

        let mut netlink_svc_mock = Box::new(MockPublicIpService::new());
        netlink_svc_mock
            .expect_get_ip()
            .times(1)
            .returning(move || Ok(local_ip));

        let kernel = DynDnsService::new("example.com", "test", dns_svc_mock, netlink_svc_mock);
        assert_eq!(
            Err(DynDnsServiceError::UnknownError),
            kernel.update_dns_if_required().await
        );
    }
}
