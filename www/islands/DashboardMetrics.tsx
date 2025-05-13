import { useEffect, useState } from "preact/hooks";

interface PanicMetric {
  version: string;
  target: string;
  trace: string;
  count: number;
  url: string;
}

export default function DashboardMetrics() {
  const [metrics, setMetrics] = useState<PanicMetric[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState("");
  const [versionFilter, setVersionFilter] = useState("");
  const [targetFilter, setTargetFilter] = useState("");

  useEffect(() => {
    async function fetchMetrics() {
      try {
        const response = await fetch("/metrics");
        if (!response.ok) {
          throw new Error(`Failed to fetch metrics: ${response.status}`);
        }
        const data = await response.json();
        setMetrics(data);
        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setLoading(false);
      }
    }

    fetchMetrics();
  }, []);

  const versions = [...new Set(metrics.map((m) => m.version))].sort((a, b) => {
    // Handle canary versions (vx.x.x+<sha>)
    const aIsCanary = a.includes("+");
    const bIsCanary = b.includes("+");

    if (aIsCanary && !bIsCanary) return 1;
    if (!aIsCanary && bIsCanary) return -1;

    const aParts = a.replace(/^v/, "").split(/[.\-+]/);
    const bParts = b.replace(/^v/, "").split(/[.\-+]/);

    for (let i = 0; i < Math.min(aParts.length, bParts.length); i++) {
      const aNum = parseInt(aParts[i], 10);
      const bNum = parseInt(bParts[i], 10);

      if (!isNaN(aNum) && !isNaN(bNum) && aNum !== bNum) {
        return bNum - aNum; // Descending order (latest first)
      }
    }

    return bParts.length - aParts.length;
  });
  const targets = [...new Set(metrics.map((m) => m.target))];

  const filteredMetrics = metrics.filter((metric) => {
    const matchesText = filter === "" ||
      metric.trace.toLowerCase().includes(filter.toLowerCase()) ||
      metric.version.toLowerCase().includes(filter.toLowerCase()) ||
      metric.target.toLowerCase().includes(filter.toLowerCase());

    const matchesVersion = versionFilter === "" ||
      metric.version === versionFilter;
    const matchesTarget = targetFilter === "" || metric.target === targetFilter;

    return matchesText && matchesVersion && matchesTarget;
  });

  return (
    <>
      <div class="flex flex-col sm:flex-row gap-4 mb-4">
        <div class="flex-1">
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Search
          </label>
          <input
            type="text"
            placeholder="Filter panics..."
            value={filter}
            onInput={(e) => setFilter((e.target as HTMLInputElement).value)}
            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Version
          </label>
          <select
            value={versionFilter}
            onChange={(e) =>
              setVersionFilter((e.target as HTMLSelectElement).value)}
            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="">All Versions</option>
            {versions.map((version) => (
              <option key={version} value={version} title={version}>
                {version.length > 15
                  ? `${version.substring(0, 12)}...`
                  : version}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Target
          </label>
          <select
            value={targetFilter}
            onChange={(e) =>
              setTargetFilter((e.target as HTMLSelectElement).value)}
            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="">All Targets</option>
            {targets.map((target) => (
              <option key={target} value={target}>{target}</option>
            ))}
          </select>
        </div>
      </div>

      {loading
        ? (
          <div class="text-center py-8">
            <div class="spinner border-4 border-gray-300 border-t-blue-500 rounded-full w-8 h-8 mx-auto animate-spin">
            </div>
            <p class="mt-2 text-gray-600">Loading metrics...</p>
          </div>
        )
        : error
        ? (
          <div class="bg-red-50 border-l-4 border-red-400 p-4 mb-4">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg
                  class="h-5 w-5 text-red-400"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div class="ml-3">
                <p class="text-sm text-red-700">
                  Error loading metrics: {error}
                </p>
              </div>
            </div>
          </div>
        )
        : (
          <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Count
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Version
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Target
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Trace
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                {filteredMetrics.length === 0
                  ? (
                    <tr>
                      <td
                        colSpan={5}
                        class="px-6 py-4 text-center text-sm text-gray-500"
                      >
                        No panic metrics found matching your filters.
                      </td>
                    </tr>
                  )
                  : (
                    filteredMetrics.map((metric) => (
                      <tr
                        key={`${metric.version}-${metric.target}-${metric.trace}`}
                      >
                        <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          {metric.count}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          <span title={metric.version} class="cursor-default">
                            {metric.version.length > 15
                              ? `${metric.version.substring(0, 12)}...`
                              : metric.version}
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {metric.target}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 max-w-xs truncate">
                          {metric.trace}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          <a
                            href={metric.url}
                            class="text-blue-600 hover:text-blue-900"
                            target="_blank"
                            rel="noopener noreferrer"
                          >
                            View Details
                          </a>
                        </td>
                      </tr>
                    ))
                  )}
              </tbody>
            </table>
          </div>
        )}
    </>
  );
}
