# if you encounter an error, the cache image might not be built yet.
# alpine build
FROM ntu_canoebot_cache:latest as BUILDER

# same workdir as cache image
WORKDIR /build/ntu_canoebot

# build actual
COPY . .
RUN touch crates/ntu_canoebot_config/build.rs
RUN cargo build --release
RUN mkdir -p bin
RUN cp target/x86_64-unknown-linux-musl/release/ntu_canoebot bin/ntu_canoebot


# compress
FROM gruebel/upx:latest as COMPRESSOR

COPY --from=BUILDER /build/ntu_canoebot/bin/ntu_canoebot /bin/ntu_canoebot
RUN upx --lzma /bin/ntu_canoebot


# alpine image
FROM alpine:latest

ENV TZ=Asia/Singapore
RUN apk add --no-cache tzdata
COPY --from=COMPRESSOR /bin/ntu_canoebot /usr/local/bin/ntu_canoebot

CMD [ "ntu_canoebot" ]
