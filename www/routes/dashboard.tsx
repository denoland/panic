import { PageProps } from "$fresh/server.ts";
import DashboardMetrics from "../islands/DashboardMetrics.tsx";

export default function Dashboard({ url }: PageProps) {
  return (
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <div class="border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div class="flex justify-between items-center">
          <h1 class="text-lg font-semibold text-gray-900">Panic Dashboard</h1>
          <a
            href="/"
            class="inline-flex items-center px-3 py-1 border border-transparent text-sm leading-4 font-medium rounded-md text-blue-700 bg-blue-100 hover:bg-blue-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Home
          </a>
        </div>
      </div>

      <div class="p-4">
        <p class="text-gray-700 mb-4">
          This dashboard shows all panic traces collected from Deno CLI users.
          Use the filters below to find specific panics by version, target
          platform, or keyword.
        </p>

        <DashboardMetrics />
      </div>
    </div>
  );
}
