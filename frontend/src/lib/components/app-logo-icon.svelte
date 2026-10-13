<script lang="ts">
  import type { SVGAttributes } from 'svelte/elements';

  let props: SVGAttributes<SVGSVGElement> = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
  <path
    fill="currentColor"
    d="M12 2L9.19 8.63 2 9.24l5.46 4.73L5.82 21 12 17.27 18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2zm0 3.24l1.93 4.58 4.95.42-3.76 3.26 1.13 4.84L12 15.4l-4.25 2.94 1.13-4.84-3.76-3.26 4.95-.42L12 5.24z"
  />
</svg>
