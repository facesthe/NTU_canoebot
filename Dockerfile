# alpine build
FROM clux/muslrust:latest as BUILDER

WORKDIR /build/ntu_canoebot
COPY . .
RUN cargo build --release
RUN mkdir -p bin
RUN cp target/x86_64-unknown-linux-musl/release/ntu_canoebot bin/ntu_canoebot


# compress
FROM gruebel/upx:latest as COMPRESSOR

COPY --from=BUILDER /build/ntu_canoebot/bin/ntu_canoebot /bin/ntu_canoebot
RUN upx /bin/ntu_canoebot


# alpine image
FROM alpine:latest

ARG teloxide_token
ARG rust_log

ENV TELOXIDE_TOKEN=$teloxide_token
ENV RUST_LOG=$rust_log

COPY --from=COMPRESSOR /bin/ntu_canoebot /usr/local/bin/ntu_canoebot

CMD [ "ntu_canoebot" ]
