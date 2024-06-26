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

# update lib.rs timestamp and build release target
RUN touch src/lib.rs && cargo build --release

# final base
FROM rust:1.66-slim

# copy templates
COPY ./templates ./templates

# copy build artifact
COPY --from=build /app/target/release/executor-rust .

# set entrypoint to run our binary
ENTRYPOINT [ "./executor-rust" ]
