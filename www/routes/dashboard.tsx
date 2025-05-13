import { PageProps } from "$fresh/server.ts";
import DashboardMetrics from "../islands/DashboardMetrics.tsx";

export default function Dashboard({ url }: PageProps) {
  return (
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <div class="border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div class="flex justify-between items-center">
          <h1 class="text-lg font-semibold text-gray-900">Panic Dashboard</h1>
        </div>
      </div>

      <div class="p-4">
        <DashboardMetrics />
      </div>
    </div>
  );
}
