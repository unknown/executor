<script lang="ts">
  import { rust } from '@codemirror/lang-rust';
  import CodeMirror from 'svelte-codemirror-editor';
  import { z } from 'zod';

  const executionResponseSchema = z.discriminatedUnion('status', [
    z.object({
      status: z.literal('Success'),
      output: z.object({
        stdout: z.string(),
        stderr: z.string()
      })
    }),
    z.object({
      status: z.literal('Error'),
      error: z.string()
    })
  ]);

  type ExecutionResponse = z.infer<typeof executionResponseSchema>;

  let code = '// Rust 1.66\nfn main() {\n    println!("Hello, world!");\n}';

  let output: ExecutionResponse | null = null;
  let loading = false;

  async function submitCode() {
    loading = true;
    try {
      const response = await fetch('/api/submit', {
        method: 'POST',
        body: JSON.stringify({ code })
      }).then((response) => {
        if (!response.ok) {
          throw new Error('API response not ok');
        }
        return response.json();
      });
      output = executionResponseSchema.parse(response);
    } catch (error) {
      output =
        error instanceof Error
          ? {
              status: 'Error',
              error: error.message
            }
          : null;
      console.error(error);
    }
    loading = false;
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
          disabled={loading}
        >
          Run
        </button>
      </form>
    </div>
  </header>
  <main class="flex min-h-0 flex-1 flex-col">
    <CodeMirror
      bind:value={code}
      lang={rust()}
      tabSize={4}
      class="min-h-0 flex-1"
      styles={{
        '&': { height: '100%' }
      }}
    />
    <div class="flex h-64 gap-2 whitespace-pre border-t p-2">
      <div class="flex min-w-0 flex-1 flex-col">
        <h2 class="mb-1 font-medium">Standard output</h2>
        <code class="prose flex-1 overflow-auto rounded-md border bg-zinc-50 p-2 text-sm">
          {stdout}
        </code>
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <h2 class="mb-1 font-medium">Standard error</h2>
        <code class="prose flex-1 overflow-auto rounded-md border bg-zinc-50 p-2 text-sm">
          {stderr}
        </code>
      </div>
    </div>
  </main>
</div>
