#########################
###### Build Image ######
#########################

FROM elixir:1.15-alpine as builder

ENV MIX_ENV=prod \
  MIX_HOME=/opt/mix \
  HEX_HOME=/opt/hex

RUN mix local.hex --force && \
  mix local.rebar --force

WORKDIR /app

COPY mix.lock mix.exs ./
COPY config config

RUN mix deps.get --only-prod && mix deps.compile

COPY lib lib

RUN mix release

#########################
##### Release Image #####
#########################

FROM erlang:26.1-alpine

# elixir expects utf8.
ENV LANG=C.UTF-8

WORKDIR /app
COPY --from=builder /app/_build/prod/rel/rathole ./
RUN chown -R nobody: /app

LABEL org.opencontainers.image.source="https://github.com/mrchypark/rathole-operator"
LABEL org.opencontainers.image.authors="mrchypark@gmail.com"

ENTRYPOINT ["/app/bin/rathole"]
CMD ["start"]
