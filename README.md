co2-exporter
============

A JSON and Prometheus interface for CO2Meter sensors, based on
[co2mon](https://github.com/lnicola/co2mon). Targets a Rasberry Pi Zero W.


## Usage

 1. Add an appropriate udev rule to e.g. `/etc/udev/rules.d/60-co2mon.rules`:

    ```
    ACTION=="add|change", SUBSYSTEMS=="usb", ATTRS{idVendor}=="04d9", ATTRS{idProduct}=="a052", MODE:="0666"
    ```
    (per [co2mon](https://github.com/lnicola/co2mon#permissions)'s documentation)
 2. Reload udev rules:

    ```bash
    udevadm control --reload
    udevadm trigger
    ```
 3. Connect a compatible sensor via USB
 4. Run `co2-exporter`

## Notes

 * Output units are temperature and CO2 PPM. The sensor outputs temperature in
   Celsius; the metric output includes a converted Fahrenheit value.
 * The sensor may occasionally time out
 *

## Sensor calibration notes

Sensor accuracy can be hit or miss, especially between multiple sensors. These
tips helped to get 2 sensors to agree within 50ppm:

 * The sensor may take several minutes to normalize after first power-on
 * Be sure to set the altitude ("ALti" mode) to your approximate elevation in
   meters
 * Manually running the calibration ("8bc" mode) outdoors or in front of an open
   window can help

## Raspberry Pi builds

Dockerfiles are provided that can generate Pi compatible builds:
 * [`Dockerfile.gnueabi`](./Dockerfile.gnueabi) for the Pi Zero (W) and
   first-gen Pi
 * [`Dockerfile.gnueabihf`](./Dockerfile.gnueabihf) for later Pi versions with
   hardware floating point support

To use, run:

```bash
docker build . -f Dockerfile.gnueabi -t co2-exporter:build
```

Once the container builds, extract the result:

```bash
mkdir -p /tmp/co2-exporter && docker run --rm -v /tmp/co2-exporter:/tmp/co2-exporter co2-exporter:build sh -c 'cp /project/target/arm-unknown-linux-gnu*/release/co2-exporter /tmp/co2-exporter/co2-exporter'
```

Copy /tmp/co2-exporter/co2-exporter to your Pi and run it.

## systemd service

Use the example systemd unit file, [`co2-exporter.service`](./co2-exporter.service):

```bash
cp co2-exporter.service /etc/systemd/system/

systemctl enable co2-exporter
systemctl start co2-exporter
```

### prometheus example

Example config in [`prometheus.yaml`](./prometheus.yaml):

```bash
docker run \
    --dns 192.168.86.1 \
    -p 9090:9090 \
    -v /tmp/prometheus.yml:/etc/prometheus/prometheus.yml \
    prom/prometheus
```

(explicit `--dns` is needed to ensure the raspi's hostname can be looked up;
substitute the value as necessary for your local dns server)
