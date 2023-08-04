# Proxy

A TCP Forward Proxy.

# Usage

```
Usage: proxy [OPTIONS]

Options:
  -c, --config <FILE>       Config file path
  -u, --upstream <ADDRESS>  Upstream address
  -b, --bind <ADDRESS>      Bind local address
  -h, --help                Print help
  -V, --version             Print version
```

# Docker

```
docker run -d -it --name proxy \
           --restart always \
           --network host \
           -v /path/to/config.toml:/app/config.toml \
           git.herf.cc/roccoon/proxy:latest
```
