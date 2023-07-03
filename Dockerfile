# alpine build
FROM clux/muslrust:latest as BUILDER

WORKDIR /build/neat
COPY . .
RUN cargo build --release
RUN mkdir -p bin
RUN cp target/x86_64-unknown-linux-musl/release/neat bin/neat


# compress
FROM gruebel/upx:latest as COMPRESSOR

COPY --from=BUILDER /build/neat/bin/neat /bin/neat
RUN upx /bin/neat


# alpine image
FROM alpine:latest

ARG teloxide_token
ARG rust_log

ENV TELOXIDE_TOKEN=$teloxide_token
ENV RUST_LOG=$rust_log

COPY --from=COMPRESSOR /bin/neat /usr/local/bin/neat

CMD [ "ntu_canoebot" ]
