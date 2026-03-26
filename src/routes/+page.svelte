<script lang="ts">
  let { data } = $props();
</script>

<div class="container mx-auto px-4 py-12 max-w-4xl">
  <div class="mb-8">
    <h1 class="text-2xl font-bold text-gray-900">FreqModa</h1>
    <p class="text-gray-500 text-sm mt-1">Transform audio with natural language</p>
  </div>

  {#if data.samples.length === 0}
    <div class="text-center py-16 text-gray-400">
      <p>No samples available.</p>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
      <table class="w-full text-sm">
        <thead class="bg-gray-50 border-b border-gray-200">
          <tr>
            <th class="text-left px-4 py-3 font-medium text-gray-600">Name</th>
            <th class="text-left px-4 py-3 font-medium text-gray-600 hidden md:table-cell">Type</th>
            <th class="text-left px-4 py-3 font-medium text-gray-600 hidden md:table-cell">Duration</th>
            <th class="px-4 py-3"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          {#each data.samples as sample}
            <tr class="hover:bg-gray-50">
              <td class="px-4 py-3 font-medium text-gray-900">{sample.title}</td>
              <td class="px-4 py-3 text-gray-500 hidden md:table-cell">
                <span class="px-2 py-0.5 rounded text-xs bg-gray-100">{sample.fileType}</span>
              </td>
              <td class="px-4 py-3 text-gray-500 font-mono hidden md:table-cell">
                {sample.duration}s
              </td>
              <td class="px-4 py-3 text-right">
                <a
                  href="/sample/{sample.streamingKey}"
                  class="bg-emerald-500 hover:bg-emerald-600 text-white px-3 py-1.5 rounded text-xs font-medium inline-block"
                >
                  Edit
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
