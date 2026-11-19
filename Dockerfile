FROM quay.io/doumanash/rust-musl:latest AS build

WORKDIR /src

COPY . /src

RUN cargo auditable build --release

FROM scratch
COPY --from=build /src/target/release/conn-test /conn-test
CMD ["/conn-test"]
