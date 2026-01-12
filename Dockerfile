# Builder stage
FROM debian:trixie-slim as builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y \
  curl \
  wget \
  file \
  libwebkit2gtk-4.1-dev \
  build-essential \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
  && apt install -y nodejs \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

COPY . .

RUN npm install
RUN npm run tauri build -- --bundles deb

# Runtime stage
FROM debian:trixie-slim

ENV DEBIAN_FRONTEND=noninteractive
ENV WEBKIT_DISABLE_DMABUF_RENDERER=1

WORKDIR /tmp

COPY --from=builder /app/src-tauri/target/release/bundle/deb/*.deb ./open-tv.deb

RUN apt update && apt install -y ./open-tv.deb \
  mesa-utils \
  libgl1-mesa-dri \
  mpv \
  ffmpeg \
  yt-dlp \
  && apt clean && rm -rf /var/lib/apt/lists/* \
  && rm open-tv.deb

CMD ["open_tv"]
