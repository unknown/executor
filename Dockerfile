# based off of https://verzettelung.com/22/12/29/
FROM rust:1.66-slim as build

# create empty project
RUN cargo new app --vcs=none
WORKDIR /app

# copy root manifests
COPY ./Cargo.lock ./Cargo.toml ./

# create dummy crates
RUN cargo new executor-rust

# copy crate manifests
COPY ./executor-rust/Cargo.toml ./executor-rust

# build dummy crates to cache dependencies
RUN cargo build --release

# copy crate sources
COPY ./executor-rust ./executor-rust

# update timestamps
RUN touch ./executor-rust/src/main.rs

# build release targets
RUN cargo build --release


# rust image
FROM rust:1.66-slim as executor-rust

# copy templates folder
COPY ./templates ./templates

# copy build artifact
COPY --from=build /app/target/release/executor-rust .

# set entrypoint to run program
ENTRYPOINT [ "./executor-rust" ]
