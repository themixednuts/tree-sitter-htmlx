<script>
	let { a, b = $bindable(), c = $bindable('default') } = $props();
	let count = $state(0);
	let doubled = $derived(count * 2);
	let expensiveCalc = $derived.by(() => {
		return heavyComputation(count);
	});
	
	$effect(() => {
		console.log('count changed:', count);
	});
	
	$effect.pre(() => {
		console.log('before DOM update');
	});
</script>

<button onclick={() => count++}>
	{count} * 2 = {doubled}
</button>
