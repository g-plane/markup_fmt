<template>
  <div v-if="label = 'a'"></div>
  <div v-if='label = "a"'></div>

  <button @click="content += &#x27;{&quot;hello&quot;: &quot;I\&#x27;m a button!&quot;}&#x27;"></button>

  <input :value="&quot;&quot;">
  <input :value="&#x27;&#x27;">
  <input :value='&quot;&quot;'>
  <input :value='&#x27;&#x27;'>
</template>
