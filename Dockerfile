# =============================================================================
# Stage 1: Builder - Compile Rust/WASM with Trunk
# =============================================================================
# Debian trixie (glibc 2.41) is required, not cosmetic: `trunk` downloads a
# prebuilt `wasm-bindgen` binary that links against GLIBC_2.38+, so on bookworm
# (glibc 2.36) the build dies with `version 'GLIBC_2.38' not found` after the
# wasm compilation has already succeeded. Only the builder needs the newer
# glibc; the runner stage below stays on bookworm.
FROM rust:1.94-trixie AS builder

# Install wasm target and trunk
# `--locked` pins trunk's deps (avoids the cssparser/lightningcss type mismatch).
RUN rustup target add wasm32-unknown-unknown \
    && cargo install --locked trunk --version 0.21.14

# Install system dependencies for sass compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo fetch --target wasm32-unknown-unknown \
    && rm -rf src \
    && rm -f target/release/konachan target/release/deps/konachan*

# Copy source code and static assets
COPY src/ src/
COPY static/ static/
COPY index.html .
COPY Trunk.toml .
COPY rustfmt.toml .

# Build with trunk for production (web feature).
# `Cargo.toml` links with `lto = "fat"` and a single codegen unit, which is the
# peak-memory hot spot of the build. The override has to sit on *this* command
# (trunk spawns cargo and inherits the environment) so the image also builds on
# memory-constrained Docker Desktop / CI runners.
RUN CARGO_PROFILE_RELEASE_LTO=off \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
    trunk build --release --features web

# =============================================================================
# Stage 2: Runner - Serve static files with nginx
# Bookworm (not trixie) because the builder only needs the newer glibc for its
# prebuilt tooling; the runtime image ships no binaries of its own.
# =============================================================================
FROM nginx:1.27-bookworm AS runner

# Remove default nginx config
RUN rm /etc/nginx/conf.d/default.conf

# Copy custom nginx config
COPY <<'EOF' /etc/nginx/conf.d/app.conf
server {
    listen 80;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    # Enable gzip compression for WASM and JS
    gzip on;
    gzip_types application/wasm application/javascript text/css text/html image/svg+xml;
    gzip_min_length 256;

    # Runtime configuration is rewritten on container start, so it must never
    # be cached (exact match takes precedence over the asset regex below).
    location = /config.js {
        add_header Cache-Control "no-store, must-revalidate";
        expires -1;
    }

    # The service worker is not content-hashed and decides what the other assets
    # do, so it must be revalidated instead of inheriting the immutable rule
    # below (exact match takes precedence over the regex).
    location = /sw.js {
        add_header Cache-Control "no-cache";
        expires -1;
    }

    # `manifest.webmanifest` has no entry in nginx's default `mime.types`, and
    # `types {}` resets the inherited table so only `default_type` applies.
    location = /manifest.webmanifest {
        types { }
        default_type application/manifest+json;
        add_header Cache-Control "no-cache";
        expires -1;
    }

    # Cache static assets aggressively
    location ~* \.(wasm|js|css|svg|png|jpg|jpeg|gif|ico|woff|woff2|ttf)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # SPA fallback - serve index.html for all routes
    location / {
        try_files $uri $uri/ /index.html;
    }
}
EOF

# Copy built static files from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Regenerate the runtime config from environment variables on container start
# (e.g. `docker run -e KONACHAN_SAFE=true ...`), so one image can serve both
# the regular and the safe mode.
COPY docker/30-konachan-config.sh /docker-entrypoint.d/30-konachan-config.sh
RUN chmod +x /docker-entrypoint.d/30-konachan-config.sh

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
