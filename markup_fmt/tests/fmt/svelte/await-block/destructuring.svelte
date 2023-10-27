{#await promise then {  a,b =''}}
    {a}
    {b}
{/await}

{#await promise}
    Loading
{:then {  a,b =''}}
    {a}
    {b}
{/await}

{#await promise then [  a,b ='']}
    {a}
    {b}
{/await}

{#await promise}
    Loading
{:then  [  a,b ='']}
    {a}
    {b}
{/await}

{#await promise catch {  a,b =''}}
    {a}
    {b}
{/await}

{#await promise}
    Loading
{:catch {  a,b =''}}
    {a}
    {b}
{/await}

{#await promise catch [  a,b ='']}
    {a}
    {b}
{/await}

{#await promise}
    Loading
{:catch  [  a,b ='']}
    {a}
    {b}
{/await}
