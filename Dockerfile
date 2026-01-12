FROM debian:bookworm-slim

ARG pkgver

ENV DEBIAN_FRONTEND=noninteractive
ENV WEBKIT_DISABLE_DMABUF_RENDERER=1

RUN apt-get update && apt-get install -y \
  wget apt \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp
RUN if dpkg --compare-versions "$pkgver" "lt" "1.9.0"; then \
    URL="https://github.com/Fredolx/open-tv/releases/download/v${pkgver}/Open.TV_${pkgver}_amd64.deb"; \
  else \
    URL="https://github.com/Fredolx/open-tv/releases/download/v${pkgver}/Fred%20TV_${pkgver}_amd64.deb"; \
  fi \
  && wget -q -O open-tv.deb "$URL" \
  && apt update \
  && apt install -y ./open-tv.deb \
  && rm open-tv.deb

CMD ["open_tv"]
