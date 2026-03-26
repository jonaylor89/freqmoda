<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<svelte:head>
  <title>FreqModa</title>
  <meta name="description" content="Transform audio with natural language" />
</svelte:head>

{@render children()}
