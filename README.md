# dyndnsd

Dyndnsd is a small microservice that allows for updating DNS records in Hetzner DNS based on your WAN IP in a OpenWRT router. Dyndnsd polls the OpenWRT ubus HTTP API for your WAN Interface IP and a Hetzer DNS Zone via the Hetzner DNS API. If your WAN IP differs from the configured DNS Record, it will update the DNS record via the Hetzner DNS API.

Dyndnsd is a small microservice and can easily be run from a Kubernetes Cluster via the provided Helm Chart. Of course, running it with docker or systemd is also possible.

The idea is inspired by [external-dns](https://github.com/kubernetes-sigs/external-dns), but not based on external IPs on Kubernetes Resources. Since we believe that in most home networks the external IP on Ingress or LoadBalancer services will not be the external IP, we explicitly use the public IP as provided by the router. The initial version of this server was aimed at running on the router itself, but this idea was abandoned in favor of this easier to develop, API-based approach.

# Running *dyndnsd*

tbd.

# Development

tbd.
