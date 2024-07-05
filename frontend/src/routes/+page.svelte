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

  let code = 'fn main() {\n    println!("Hello, world!");\n}';

  let output: ExecutionResponse | null = null;
  let loading = false;

  async function submitCode() {
    loading = true;
    try {
      const response = await fetch('/api/submit', {
        method: 'POST',
        body: JSON.stringify({ code })
      }).then((response) => response.json());
      loading = false;
      output = executionResponseSchema.parse(response);
    } catch (error) {
      loading = false;
      console.error(error);
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

<div class="flex min-h-screen flex-col">
  <header class="p-2">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-medium">Executor</h1>
      <form>
        <button
          class="rounded-md bg-zinc-100 px-3 py-2 text-sm"
          on:click={submitCode}
          disabled={loading}
        >
          Submit
        </button>
      </form>
    </div>
  </header>
  <main class="flex flex-col">
    <CodeMirror bind:value={code} lang={rust()} tabSize={4} />
    <div class="prose whitespace-pre">
      <div>
        <h2>stdout</h2>
        <code>{stdout}</code>
      </div>
      <div>
        <h2>stderr</h2>
        <code>{stderr}</code>
      </div>
    </div>
  </main>
</div>
