# based off of https://verzettelung.com/22/12/29/
FROM rust:1.66 as build

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
FROM debian:bullseye-slim

# copy build artifact
COPY --from=build /app/target/release/backend .

# set entrypoint to run our binary
ENTRYPOINT [ "./backend" ]
