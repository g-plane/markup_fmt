{#each { length: 10 }}
  <p>fails</p>
{/each}

{#each { length: 10 }, index}
  <p>fails</p>
{/each}
