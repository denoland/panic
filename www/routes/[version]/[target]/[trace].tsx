import { Handler, PageProps } from "$fresh/server.ts";

import { symbolicate } from "../../../../wasm/lib/rs_lib.js";

const kv = await Deno.openKv();

type Trace = {
  demangledName: string;
  name: string;
  language: string;
  fullPath: string;
  line: number;
}[][];

function formatStackTraceMarkdown(trace: Trace): string {
  let markdown = "stack backtrace:\n";
  trace.forEach((frame, frameIndex) => {
    frame.forEach((f, index) => {
      const githubLink = getGithubLink(f.fullPath, f.line);

      if (githubLink) {
        markdown += `   - [\`${f.demangledName}\`](${githubLink})\n`;
      } else {
        markdown += `   - \`${f.demangledName}\`\n`;
      }
    });
  });
  return markdown;
}

function createGithubIssueUrl(
  trace: Trace,
  version: string,
  target: string,
  trace_str: string,
  url: string,
): string {
  const body = encodeURIComponent(
    `Deno ${version} on ${target}.\n\n` +
      `A panic occurred in Deno.\n\n` +
      formatStackTraceMarkdown(trace) +
      `\n\n<sub>View trace: [here](${url})</sub>`,
  );
  return `https://github.com/denoland/deno/issues/new?body=${body}&labels=bug`;
}
function getGithubLink(path: string, line: number): string | null {
  // Handle Rust standard library paths
  const rustcMatch = path.match(/\/rustc\/([a-f0-9]+)\/library\/(.*)/);
  if (rustcMatch) {
    const [_, commit, filepath] = rustcMatch;
    return `https://github.com/rust-lang/rust/blob/${commit}/library/${filepath}#L${line}`;
  }

  // Handle Deno repository paths
  const denoMatch = path.match(/\/gh\/deno\/(.*)/);
  if (denoMatch) {
    const [_, filepath] = denoMatch;
    // Using main branch as default, but this could be made configurable
    return `https://github.com/denoland/deno/blob/main/${filepath}#L${line}`;
  }

  // Handle panic repository paths (assuming it's your repository)
  const panicMatch = path.match(/\/gh\/panic\/(.*)/);
  if (panicMatch) {
    const [_, filepath] = panicMatch;
    // Using main branch as default
    return `https://github.com/denoland/panic/blob/main/${filepath}#L${line}`;
  }

  return null;
}

function Stacktrace({ trace, ghUrl }: { trace: Trace; ghUrl: string }) {
  return (
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <div class="border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div class="flex justify-between items-center">
          <h1 class="text-lg font-semibold text-gray-900">Stacktrace</h1>
          <a
            href={ghUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <svg class="h-4 w-4 mr-1.5" fill="currentColor" viewBox="0 0 24 24">
              <path
                fill-rule="evenodd"
                d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                clip-rule="evenodd"
              />
            </svg>
            Create Issue
          </a>
        </div>
      </div>
      <ul class="divide-y divide-gray-100">
        {trace.map((frame, frameIndex) => (
          <li class="hover:bg-gray-50 transition-colors duration-150 ease-in-out">
            <div class="px-4 py-2">
              <div class="space-y-1">
                {frame.map((f, index) => (
                  <div class="space-y-0.5">
                    <div class="flex items-center space-x-2">
                      <span class="text-gray-500 text-xs font-mono">
                        {frameIndex}.{index}
                      </span>
                      <p class="text-sm text-indigo-600 font-medium font-mono">
                        {f.demangledName}
                      </p>
                      {f.language && (
                        <span class="inline-flex items-center px-1.5 py-0.5 rounded-md text-xs font-medium bg-blue-50 text-blue-700">
                          {f.language}
                        </span>
                      )}
                    </div>
                    <div class="flex items-center space-x-1.5 ml-4">
                      <svg
                        class="h-3.5 w-3.5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                      </svg>
                      <p class="text-xs text-gray-600 font-mono">
                        {(() => {
                          const githubLink = getGithubLink(f.fullPath, f.line);
                          if (githubLink) {
                            return (
                              <a
                                href={githubLink}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="hover:text-indigo-600 hover:underline"
                              >
                                {f.fullPath}:<span class="text-gray-900">
                                  {f.line}
                                </span>
                              </a>
                            );
                          }
                          return (
                            <span>
                              {f.fullPath}:<span class="text-gray-900">
                                {f.line}
                              </span>
                            </span>
                          );
                        })()}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}

async function getSymcache(version, target) {
  if (Deno.env.get("DENO_SYMCACHE")) {
    return Deno.readFileSync(Deno.env.get("DENO_SYMCACHE"));
  }

  let type = "release";
  if (version.includes("+")) {
    const [v, hash] = version.split("+");
    type = "canary";

    // Resolve the full sha from the short hash
    const res = await fetch(
      `https://api.github.com/repos/denoland/deno/commits/${hash}`,
    );
    if (!res.ok) {
      throw new Error("Failed to fetch commit");
    }

    const sha = await res.json();
    version = sha.sha;
  }

  const url =
    `https://storage.googleapis.com/dl.deno.land/${type}/${version}/deno-${target}.symcache`;
  const zip = await fetch(url);

  if (zip.status === 404) {
    throw new Error("Debug info not found");
  }

  if (!zip.ok) {
    throw new Error("Failed to fetch debug info");
  }

  return new Uint8Array(await zip.arrayBuffer());
}

export const handler: Handlers = {
  async GET(req, ctx) {
    const { version, target, trace: trace_str } = ctx.params;

    const key = [version, target, trace_str];

    const res = await kv.get(["trace", ...key]);
    let trace;

    if (res.value) {
      trace = res.value;
      await kv.atomic().sum(["metric", ...key], 1n).commit();
    } else {
      try {
        const symcache = await getSymcache(version, target);

        trace = symbolicate(trace_str, symcache);

        await kv
          .atomic()
          .set(["trace", ...key], trace)
          .sum(["metric", ...key], 1n)
          .commit();
      } catch (e) {
        console.error(e);
        return Response.json(e.message, { status: 500 });
      }
    }

    const ghUrl = createGithubIssueUrl(
      trace,
      version,
      target,
      trace_str,
      req.url,
    );

    const resp = await ctx.render({
      trace,
      ghUrl,
    });
    return resp;
  },
};

export default function GreetPage(props: PageProps) {
  const { ghUrl, trace } = props.data;

  return (
    <main>
      <Stacktrace trace={trace} ghUrl={ghUrl} />
    </main>
  );
}
