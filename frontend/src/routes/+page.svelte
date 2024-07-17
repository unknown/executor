<script lang="ts">
  import { rust } from '@codemirror/lang-rust';
  import { Dialog } from 'bits-ui';
  import { Loader2, Play, Settings } from 'lucide-svelte';
  import { PaneGroup, Pane, PaneResizer } from 'paneforge';
  import CodeMirror from 'svelte-codemirror-editor';
  import { submitSchema, executionOutputSchema } from '$lib/schemas';
  import type { ExecutionOutputResponse, SubmitResponse } from '$lib/schemas';

  let code = '// Rust 1.66\nfn main() {\n    println!("Hello, world!");\n}';

  let pending = false;
  let outputResponse: ExecutionOutputResponse | null = null;
  let submitResponse: SubmitResponse | null = null;
  let errorMessage: string | null = null;

  async function submitCode() {
    pending = true;
    outputResponse = null;
    submitResponse = null;
    errorMessage = null;
    try {
      submitResponse = await fetch('/api/submit', {
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

      if (submitResponse?.status !== 'Success') {
        pending = false;
        return;
      }

      pollOutput();
    } catch (error) {
      errorMessage = String(error);
      pending = false;
    }
  }

  async function pollOutput() {
    pending = true;
    if (
      submitResponse?.status === 'Success' &&
      submitResponse.job_id !== null &&
      submitResponse.job_name !== null
    ) {
      const { job_name, job_id } = submitResponse;
      try {
        outputResponse = await fetch(`/api/status/${job_name}/${job_id}`, {
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
        errorMessage = String(error);
      }

      if (outputResponse?.status === 'Success' && outputResponse.output.pending) {
        setTimeout(pollOutput, 1000);
        return;
      }
    }

    pending = false;
  }

  $: stdout = outputResponse?.status === 'Success' ? outputResponse.output.stdout : '';
  $: stderr =
    errorMessage !== null
      ? errorMessage
      : outputResponse?.status === 'Success'
        ? outputResponse.output.stderr
        : outputResponse?.status === 'Error'
          ? outputResponse.error
          : '';
</script>

<div class="flex h-screen flex-col">
  <header class="flex-shrink-0 border-b p-2">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="font-medium">Executor</h1>
      </div>
      <div>
        <form>
          <button
            class="inline-flex items-center justify-center rounded-md bg-green-500 px-3.5 py-2 text-sm font-medium text-white transition-colors hover:bg-green-600 disabled:bg-green-400"
            on:click={submitCode}
            disabled={pending}
          >
            {#if pending}
              <Loader2 class="mr-2 size-4 animate-spin" />
            {:else}
              <Play class="mr-2 size-4" />
            {/if}
            Run
          </button>
        </form>
      </div>
      <div>
        <Dialog.Root>
          <Dialog.Trigger
            class="inline-flex items-center justify-center rounded-md px-2 py-2 text-sm transition-colors hover:bg-zinc-200"
          >
            <Settings class="size-4" />
          </Dialog.Trigger>
          <Dialog.Portal>
            <Dialog.Overlay class="fixed inset-0 z-50 bg-black/80" />
            <Dialog.Content
              class="fixed left-[50%] top-[50%] z-50 w-full max-w-xl translate-x-[-50%] translate-y-[-50%] rounded-md border bg-white p-2"
            >
              <Dialog.Title>Settings</Dialog.Title>
              <Dialog.Description>Test</Dialog.Description>
            </Dialog.Content>
          </Dialog.Portal>
        </Dialog.Root>
      </div>
    </div>
  </header>
  <main class="flex min-h-0 flex-1 flex-col bg-zinc-100">
    <PaneGroup direction="vertical" class="p-2">
      <Pane defaultSize={70} minSize={20} class="rounded-md border bg-white">
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
          class="my-1 h-0.5 w-6 rounded-full bg-zinc-300 group-hover:w-full group-hover:bg-blue-500 group-active:w-full group-active:bg-blue-600"
        />
      </PaneResizer>
      <Pane defaultSize={30} minSize={20} collapsible class="rounded-md border bg-white">
        <div class="flex h-full flex-col gap-3 p-3 md:flex-row">
          <div class="flex min-h-0 min-w-0 flex-1 flex-col">
            <h2 class="mb-1 font-medium text-gray-600">Standard output</h2>
            <code class="flex-1 overflow-auto whitespace-pre rounded-md bg-zinc-100 p-2 text-sm">
              {stdout}
            </code>
          </div>
          <div class="flex min-h-0 min-w-0 flex-1 flex-col">
            <h2 class="mb-1 font-medium text-gray-600">Standard error</h2>
            <code class="flex-1 overflow-auto whitespace-pre rounded-md bg-zinc-100 p-2 text-sm">
              {stderr}
            </code>
          </div>
        </div>
      </Pane>
    </PaneGroup>
  </main>
</div>
