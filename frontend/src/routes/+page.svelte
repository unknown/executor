<script lang="ts">
  import { rust } from '@codemirror/lang-rust';
  import { PaneGroup, Pane, PaneResizer } from 'paneforge';
  import CodeMirror from 'svelte-codemirror-editor';
  import { submitSchema, executionOutputSchema } from '$lib/schemas';
  import type { ExecutionOutputResponse } from '$lib/schemas';

  let code = '// Rust 1.66\nfn main() {\n    println!("Hello, world!");\n}';

  let pending = false;
  let output: ExecutionOutputResponse | null = null;
  let job_id: string | null = null;
  let job_name: string | null = null;
  let interval: number | null = null;

  async function submitCode() {
    pending = true;
    output = null;
    job_id = null;
    job_name = null;
    try {
      const response = await fetch('/api/submit', {
        method: 'POST',
        body: JSON.stringify({ code })
      })
        .then((response) => {
          if (!response.ok) {
            throw new Error('API response not ok');
          }
          return response.json();
        })
        .then((response) => submitSchema.parse(response));

      if (response.status === 'Error') {
        pending = false;
        return;
      }

      job_id = response.job_id;
      job_name = response.job_name;
      interval = setInterval(pollOutput, 1000);
    } catch (error) {
      console.error(error);
      pending = false;
      output =
        error instanceof Error
          ? {
              status: 'Error',
              error: error.message
            }
          : null;
    }
  }

  async function pollOutput() {
    if (interval === null) {
      return;
    }

    if (job_id === null || job_name === null) {
      clearInterval(interval);
      return;
    }

    try {
      output = await fetch(`/api/status/${job_name}/${job_id}`, {
        method: 'GET'
      })
        .then((response) => {
          if (!response.ok) {
            throw new Error('API response not ok');
          }
          return response.json();
        })
        .then((response) => executionOutputSchema.parse(response));
    } catch (error) {
      console.error(error);
      output =
        error instanceof Error
          ? {
              status: 'Error',
              error: error.message
            }
          : null;
    }

    if (
      output?.status === 'Error' ||
      (output?.status === 'Success' && output.output.pending === false)
    ) {
      pending = false;
      clearInterval(interval);
    }
  }

  $: stdout = output?.status === 'Success' ? output.output.stdout : '';
  $: stderr =
    output?.status === 'Success'
      ? output.output.stderr
      : output?.status === 'Error'
        ? output.error
        : '';
</script>

<div class="flex h-screen flex-col">
  <header class="flex-shrink-0 border-b p-2">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-medium">Executor</h1>
      <form>
        <button
          class="rounded-md bg-green-500 px-3.5 py-2 text-sm font-medium text-white hover:bg-green-600 disabled:bg-green-300"
          on:click={submitCode}
          disabled={pending}
        >
          Run
        </button>
      </form>
    </div>
  </header>
  <main class="flex min-h-0 flex-1 flex-col">
    <PaneGroup direction="vertical" class="p-2">
      <Pane defaultSize={70} minSize={20} class="rounded-md border">
        <CodeMirror
          bind:value={code}
          lang={rust()}
          tabSize={4}
          class="h-full"
          styles={{
            '&': { height: '100%' }
          }}
        />
      </Pane>
      <PaneResizer class="group relative flex items-center justify-center">
        <div
          class="my-1 h-0.5 w-6 rounded-full bg-zinc-200 group-hover:w-full group-hover:bg-blue-500 group-active:w-full group-active:bg-blue-600"
        />
      </PaneResizer>
      <Pane defaultSize={30} minSize={20} collapsible class="flex gap-2">
        <div class="flex min-w-0 flex-1 flex-col">
          <h2 class="mb-1 font-medium text-gray-600">Standard output</h2>
          <code
            class="prose flex-1 overflow-auto whitespace-pre rounded-md border bg-zinc-50 p-2 text-sm"
          >
            {stdout}
          </code>
        </div>
        <div class="flex min-w-0 flex-1 flex-col">
          <h2 class="mb-1 font-medium text-gray-600">Standard error</h2>
          <code
            class="prose flex-1 overflow-auto whitespace-pre rounded-md border bg-zinc-50 p-2 text-sm"
          >
            {stderr}
          </code>
        </div>
      </Pane>
    </PaneGroup>
  </main>
</div>
