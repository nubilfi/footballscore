FROM rust:alpine3.18 as buildstage

RUN apk add musl-dev

RUN mkdir -p /footballscore && \
    cd /footballscore && \
    cargo install footballscore --root=/footballscore

FROM alpine:3.18

COPY --from=buildstage /footballscore/bin/* /usr/bin/

STOPSIGNAL SIGTERM

CMD ["footballscore"]
