<script lang="ts">
  import CodeMirror from 'svelte-codemirror-editor';
  import { rust } from '@codemirror/lang-rust';

  let code = 'fn main() {\n    println!("Hello, world!");\n}';

  let output: string | null = null;
  let loading = false;

  async function submitCode() {
    loading = true;
    try {
      const response = await fetch('/api/submit', {
        method: 'POST',
        body: JSON.stringify({ code })
      });
      loading = false;
      const json = await response.json();
      output = JSON.stringify(json);
    } catch (error) {
      loading = false;
      console.error(error);
    }
  }
</script>

<main class="flex flex-col">
  <header class="p-2">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-medium">Executor</h1>
      <form>
        <button
          class="bg-zinc-100 px-3 py-2 text-sm rounded-md"
          on:click={submitCode}
          disabled={loading}
        >
          Submit
        </button>
      </form>
    </div>
  </header>
  <CodeMirror bind:value={code} lang={rust()} tabSize={4} />
  {#if output !== null}
    <div>{output}</div>
  {/if}
</main>
