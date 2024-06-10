# based off of https://verzettelung.com/22/12/29/
FROM rust:1.66-slim as build

# create empty project
RUN cargo new app --vcs=none
WORKDIR /app

# copy root manifests
COPY ./Cargo.lock ./Cargo.toml ./

# create dummy crates
RUN cargo new builder
RUN cargo new runner

# copy crate manifests
COPY ./builder/Cargo.toml ./builder
COPY ./runner/Cargo.toml ./runner

# build dummy crates to cache dependencies
RUN cargo build --release

# copy crate sources
COPY ./builder ./builder
COPY ./runner ./runner

# update timestamps
RUN touch ./builder/src/main.rs
RUN touch ./runner/src/main.rs

# build release targets
RUN cargo build --release


# builder base
FROM rust:1.66-slim as builder

# copy templates folder
COPY ./templates ./templates

# copy build artifact
COPY --from=build /app/target/release/builder .

# set entrypoint to run program
ENTRYPOINT [ "./builder" ]


# runner base
FROM debian:bullseye-slim as runner

# copy build artifact
COPY --from=build /app/target/release/runner .

# set entrypoint to run program
ENTRYPOINT [ "./runner" ]
