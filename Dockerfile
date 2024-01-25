FROM rust:alpine as builder
WORKDIR /
RUN apk add nasm libwebp-dev clang-static musl-dev pkgconf libdav1d dav1d git meson ninja
COPY . .
RUN SYSTEM_DEPS_LINK=static SYSTEM_DEPS_BUILD_INTERNAL=always cargo build --release

FROM scratch
COPY --from=builder /target/release/image_conversion /
CMD ["/image_conversion"]
