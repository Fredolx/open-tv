FROM debian:bookworm-slim

ARG pkgver
ENV DEBIAN_FRONTEND=noninteractive
ENV WEBKIT_DISABLE_DMABUF_RENDERER=1

RUN apt-get update && apt-get install -y \
  wget apt \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp
RUN wget -q https://github.com/Fredolx/open-tv/releases/download/v${pkgver}/Open.TV_${pkgver}_amd64.deb \
  && apt update \
  && apt install -y ./Open.TV_${pkgver}_amd64.deb \
  && rm Open.TV_${pkgver}_amd64.deb

CMD ["open_tv"]
