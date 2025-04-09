import { Handlers } from "$fresh/server.ts";

const kv = await Deno.openKv(Deno.env.get("DENO_KV_DATABASE_URL"));

export const handler: Handlers = {
  async GET() {
    const entries = await kv.list({ prefix: ["metric"] });
    const traces = [];

    for await (const entry of entries) {
      traces.push({
        version: entry.key[1],
        target: entry.key[2],
        trace: entry.key[3],
        count: Number(entry.value),
      });
    }

    return Response.json(traces);
  },
};
