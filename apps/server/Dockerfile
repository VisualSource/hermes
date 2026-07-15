FROM rust:1.90.0 as build 

WORKDIR /usr/src/hermes 
COPY . .

ARG GIT_HASH 
RUN --mount=type=cache,target=/usr/src/hermes/target \
	--mount=type=cache,target=/usr/local/cargo,from=rust:1.90.0,source=/usr/local/cargo \
	rustup target add x86_64-unknown-linux-musl && SQLX_OFFLINE=true cargo build --profile release --package hermes-server --target x86_64-unknown-linux-musl

FROM build AS artifacts

RUN --mount=type=cache,target=/usr/src/hermes/target \
	mkdir /hermes \
	&& cp /usr/src/labrinth/target/release/hermes /hermes/hermes 
# \
#	&& cp -r /usr/src/labrinth/apps/labrinth/migrations /labrinth \
#	&& cp -r /usr/src/labrinth/apps/labrinth/assets /labrinth

FROM alpine:3.23

LABEL org.opencontainers.image.source=https://github.com/VisualSource/hermes
LABEL org.opencontainers.image.title=Hermes
LABEL org.opencontainers.image.description="Hermes API"
LABEL org.opencontainers.image.licenses=MIT

RUN apk update && apk add ca-certificates && apk cache clean

COPY --from=artifacts /hermes /hermes 

EXPOSE 4733
CMD ["/hermes/hermes"]

# https://medium.com/@emilia.jaser/containerise-rust-applications-on-ubuntu-alpine-with-github-actions-or-docker-builders-9378a02b98fd
# https://users.rust-lang.org/t/building-executable-for-alpine-linux/13568
# https://github.com/modrinth/code/blob/main/apps/labrinth/Cargo.toml