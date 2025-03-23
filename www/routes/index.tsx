export default function Home() {
  return (
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <div class="border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div class="flex justify-between items-center">
          <h1 class="text-lg font-semibold text-gray-900">Deno Symbolicator</h1>
        </div>
      </div>
      <div class="p-4">
        <p class="text-gray-700 mb-4">
          This tool converts URL-safe panic traces from Deno into readable stack
          traces. Just paste in a trace string, and it’ll symbolicate it for
          you.
        </p>
        <p class="text-gray-700 mb-4">
          Check out an{" "}
          <a
            href="/v2.2.5/aarch64-apple-darwin/gszD49_B4utrqB4vrrqBozirqB49prqBwjkwqBw_jBg31Cw5tCg5sDoo3pqB41sDgkkB"
            class="text-blue-500"
          >
            example stack trace
          </a>.
        </p>

        <h2 class="text-lg font-semibold text-gray-900 mb-2">
          Why does this exist?
        </h2>
        <p class="text-gray-700">
          Panics in Deno are rare—but when they happen, understanding them is
          crucial. Including debug info directly in binaries makes them bulky.
          This tool lets you symbolicate traces remotely, so you can keep your
          binaries slim without losing observability.
        </p>
      </div>
    </div>
  );
}
