<script lang="ts">
import type { Snippet } from "svelte";
import "./layout.scss";

let { children }: { children: Snippet } = $props();
</script>

{@render children()}
