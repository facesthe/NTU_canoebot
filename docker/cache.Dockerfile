# docker build -t ntu_canoebot_cache -f docker/cache.Dockerfile .

FROM clux/muslrust:stable

WORKDIR /build/ntu_canoebot
# copy only Cargo manifest files (refer to dockerignore)
COPY . .

RUN ls -lah
RUN ls crates/*

# populate with empty main + lib + build files
# cargo will figure out what needs to be built
RUN touch lib.rs
RUN echo "fn main() {}" > main.rs
RUN echo "fn main() {}" > build.rs
RUN for dir in crates/*/; do mkdir -p $dir/src && cp *.rs $dir/src && cp *.rs $dir; done

RUN cargo build --release
RUN find crates/ -name "*.rs" -type f -delete
