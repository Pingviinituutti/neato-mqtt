FROM gcr.io/distroless/static@sha256:6706c73aae2afaa8201d63cc3dda48753c09bcd6c300762251065c0f7e602b25
COPY target/x86_64-unknown-linux-musl/release/neato-mqtt /usr/local/bin/neato-mqtt
CMD ["neato-mqtt"]
