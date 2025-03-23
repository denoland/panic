import { type PageProps } from "$fresh/server.ts";
export default function App({ Component }: PageProps) {
  return (
    <html>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Deno symbolicator</title>
        <link rel="stylesheet" href="/styles.css" />
      </head>
      <body>
        <div class="min-h-screen bg-gray-50">
          <div class="max-w-3xl mx-auto px-4 py-4">
            <Component />
          </div>
        </div>
      </body>
    </html>
  );
}
