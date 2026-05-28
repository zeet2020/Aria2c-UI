<template>
  <div class="logo">
    <svg xmlns="http://www.w3.org/2000/svg" :width="width" :height="height" viewBox="0 0 62 14">
      <text
        x="0"
        y="12"
        font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
        font-size="14"
        font-weight="700"
        letter-spacing="1.5"
      >AUI</text>
    </svg>
  </div>
</template>

<script>
  export default {
    name: 'mo-logo',
    props: {
      width: {
        type: Number,
        default: 62
      },
      height: {
        type: Number,
        default: 14
      }
    }
  }
</script>

<style lang="scss">
.logo {
  display: inline-block;
  margin: 0;
  padding: 0;
  color: $--app-logo-color;
  svg {
    fill: currentColor;
  }
}
</style>
