{#each items as item, index (item.id)}
	<li>{index}: {item.name}</li>
{:else}
	<p>No items</p>
{/each}
