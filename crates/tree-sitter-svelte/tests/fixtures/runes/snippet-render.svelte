{#snippet button(text, onclick)}
	<button onclick={onclick}>
		{text}
	</button>
{/snippet}

{@render button("Click me", handleClick)}
