/** @jsx h */
import { h, ssr } from "https://crux.land/nanossr@0.0.4";
import { create_symcache, symbolicate } from "../wasm/lib/rs_lib.js";

const ROUTE = new URLPattern({ pathname: '/:version/:target/:trace_str' });

function App({ children }) {
  return (
    <div class="min-h-screen">
      {children}
    </div>
  );
}

type Trace = {
  demangledName: string;
  name: string;
  language: string;
  fullPath: string;
  line: number;
}[][];

function Stacktrace({ trace }: { trace : Trace }) {
  return (
    <div class="p-4">
      <h1 class="text-2xl font-bold">Stacktrace</h1>
      <ul class="divide-y divide-gray-200">
	{trace.map((frame) => (
	  <li class="py-4">
	    <div class="flex space-x-3">
	      <div class="flex-1 space-y-1">
	      	{frame.map((f) => (
		  <div>
		    <p class="text-sm font-medium text-indigo-600">{f.demangledName}</p>
		    <p class="text-sm text-gray-500">{f.fullPath}:{f.line}</p>
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

const render = (component) => ssr(() => <App>{component}</App>);

async function getSymcache(version, target) {
  const url = `https://storage.googleapis.com/dl.deno.land/release/${version}/deno-${target}.symcache`;
  const zip = await fetch(url);

  return await zip.arrayBuffer();
}

const kv = await Deno.openKv("symcache");

// http://localhost:8000/2.2.3/aarch64-apple-darwin/4mzDo4sCoi-rqBoj8rqB4mzrqBox6rqBg30wqB4k4BwnjD4kvDguvDgqkqqB4qvDop4B
Deno.serve(async (req) => {
  const match = ROUTE.exec(req.url);
  if (!match) {
    return new Response('Not Found', { status: 404 });
  }

  const { version, target, trace_str } = match.pathname.groups;
  
  const res = await kv.get([version, target, trace_str]);
  if (res.value) {
    return render(<Stacktrace trace={res.value} />);
  }

  const symcache = create_symcache(await getSymcache(version, target));
  const trace = symbolicate(trace_str, symcache);

  await kv.set([version, target, trace_str], trace);

  return render(<Stacktrace trace={trace} />);
});
