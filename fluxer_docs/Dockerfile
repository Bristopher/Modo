# SPDX-License-Identifier: AGPL-3.0-or-later

FROM python:3.13-alpine AS builder

WORKDIR /src/fluxer_docs

COPY requirements.txt requirements.txt
RUN python -m pip install --no-cache-dir --upgrade pip \
	&& python -m pip install --no-cache-dir -r requirements.txt

COPY zensical.toml zensical.toml
COPY docs docs
COPY scripts scripts
RUN zensical build --clean \
	&& python scripts/write_csp_header.py site /tmp/security-headers.conf

FROM nginxinc/nginx-unprivileged:1.29-alpine AS runtime

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /tmp/security-headers.conf /etc/nginx/conf.d/security-headers.conf
COPY --from=builder /src/fluxer_docs/site /usr/share/nginx/html

EXPOSE 8080
