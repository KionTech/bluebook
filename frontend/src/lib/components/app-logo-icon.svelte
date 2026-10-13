<script lang="ts">
  import type { SVGAttributes } from 'svelte/elements';

  let props: SVGAttributes<SVGSVGElement> = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  {...props}
>
  <path d="M12 2L4 7v10l8 5 8-5V7l-8-5Z" />
  <path d="M12 22V12" />
  <path d="M20 7l-8 5" />
  <path d="M4 7l8 5" />
</svg>
