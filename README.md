co2
===

A JSON and Prometheus interface for CO2Meter sensors, based on
[co2mon][https://github.com/vfilimonov/co2mon]. Targets a Rasberry Pi Zero W.


### raspberry pi builds

Two Dockerfiles can be 


### prometheus example

Example config in [`prometheus.yaml`](./prometheus.yaml):

```bash
docker run \
    --dns 192.168.86.1 \
    -p 9090:9090 \
    -v /tmp/prometheus.yml:/etc/prometheus/prometheus.yml \
    prom/prometheus
```

(explicit `--dns` is needed to ensure the raspi's hostname can be looked up)
