FROM denoland/deno:2.1.8
WORKDIR /app

COPY www/ www/
COPY wasm/lib/ wasm/lib/

WORKDIR /app/www

RUN deno task build

# Warmup caches
RUN timeout 2s deno run -A --cached-only main.ts || true

CMD ["run", "-A", "--cached-only", "main.ts"]

