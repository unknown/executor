# based off of https://verzettelung.com/22/12/29/
FROM rust:1.66-slim as build

# create empty project
RUN cargo new app --lib --vcs=none
WORKDIR /app

# copy manifests
COPY ./Cargo.lock ./Cargo.toml ./

# build dependencies to cache them
RUN cargo build --release --lib

# copy source files
COPY ./src ./src

# build release target
RUN cargo build --release

# final base
FROM debian:bullseye-slim

# copy build artifact
COPY --from=build /app/target/release/executor .

# set entrypoint to run our binary
ENTRYPOINT [ "./executor" ]
