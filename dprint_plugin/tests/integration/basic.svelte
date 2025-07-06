<script lang="ts">
export let msg:string=''

function generateHtml(data:unknown) {
  return ''
}
</script>

<div class={ `123` } title="{ `456` }">{  msg }</div>

<div>
{@html generateHtml( { key:'value' } ) }
</div>

  {#await   fetch( '', {} )   }...{:then {status   }}...{:catch    error  }...{/await}
  {#await   fetch( '', {} ) then name}...{/await}
  {#await   fetch( '', {} ) catch {message}}...{/await}

{#each [{value:1},{value:2}] as item,index (item   .  value)}...{/each}
{#each [{value:1},{value:2},{value:3},{value:4}] as {    value},index (value)}...{/each}
{#each [1 ,2 ,3]  as  Array < number > as value}{/each}
{#each [] as Array< {[P  in keyof  T as A|B]:T[ P ] }> as binding }{/each}

{#if    Math.random(  )>0.5}...{:else if Date.now(   )>0    }...{/if}

{#key 1+2}s{/key}

<button class="{}" on:click={}>{}{/**/}</button>

{#snippet foo(param: string)}
{/snippet}

<canvas
	width={32}
	height={32}
    {
      @attach canvas=>{
      const context=canvas.getContext('2d');

      $effect(()=>{
        context.fillStyle = color;
        context.fillRect(0, 0, canvas.width, canvas.height);
      });
    }}
></canvas>

<ul>
        {#each         [
          [`Model Version`, model.model_version],
          [`Model Type`, model.model_type],]}
          {/each}
          </ul>
