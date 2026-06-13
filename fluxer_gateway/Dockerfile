FROM erlang:28-slim AS build

ARG LOGGER_LEVEL=info

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y --no-install-recommends \
	git \
	curl \
	make \
	gcc \
	g++ \
	libc6-dev \
	gettext-base \
	ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://github.com/erlang/rebar3/releases/download/3.24.0/rebar3 -o /usr/local/bin/rebar3 && \
	echo "d2d31cfb98904b8e4917300a75f870de12cb5167cd6214d1043e973a56668a54  /usr/local/bin/rebar3" | sha256sum -c - && \
	chmod +x /usr/local/bin/rebar3

COPY rebar.config rebar.lock* ./
RUN rebar3 compile --deps_only

COPY . .
RUN LOGGER_LEVEL=${LOGGER_LEVEL} envsubst < config/sys.config.template > config/sys.config && \
	rebar3 as prod release

FROM erlang:28-slim

WORKDIR /opt/fluxer_gateway

RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates \
	curl \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/app/_build/prod/rel/fluxer_gateway .
COPY scripts/docker_entrypoint.sh /opt/fluxer_gateway/bin/docker_entrypoint.sh

# Strip any CR that a Windows checkout may have baked into the script. A CRLF
# shebang makes the kernel exec `sh\r` and the container dies with
# `env: 'sh\r': No such file or directory`.
RUN sed -i 's/\r$//' /opt/fluxer_gateway/bin/docker_entrypoint.sh && \
	chmod +x /opt/fluxer_gateway/bin/docker_entrypoint.sh && \
	useradd -r -s /sbin/nologin -d /opt/fluxer_gateway fluxer && \
	chown -R fluxer:fluxer /opt/fluxer_gateway

USER fluxer

EXPOSE 8080 8081

ENTRYPOINT ["/opt/fluxer_gateway/bin/docker_entrypoint.sh"]
