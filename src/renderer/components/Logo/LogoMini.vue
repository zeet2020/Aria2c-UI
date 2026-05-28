<template>
  <h1 class="logo-mini">
    <a
      href="#"
      class="logo-link"
      title="Aria2 UI on GitHub"
      @click.prevent="openRepo"
    >
      <!-- AUI "A" mark: white A legs + cyan download chevron -->
      <svg class="logo-mark" xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 32 32" fill="none">
        <path d="M16 5 L7 27" stroke="#FFFFFF" stroke-width="3.2" stroke-linecap="round"/>
        <path d="M16 5 L25 27" stroke="#FFFFFF" stroke-width="3.2" stroke-linecap="round"/>
        <path d="M9.5 18 L16 22 L22.5 18" stroke="#27D3FF" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <span class="logo-text">AUI</span>
    </a>
  </h1>
</template>

<script>
  import { shell } from '@electron/remote'

  export default {
    name: 'mo-logo-mini',
    methods: {
      openRepo () {
        shell.openExternal('https://github.com/zeet2020/Aria2c-UI')
      }
    }
  }
</script>

<style lang="scss">
.logo-mini {
  margin: 0;
  padding: 0;
  width: 100%;
  text-align: center;
  .logo-link {
    display: block;
    outline: none;
    text-decoration: none;
    cursor: pointer;
  }
  .logo-mark {
    display: block;
    margin: 0 auto 2px;
  }
  .logo-text {
    display: block;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 1px;
    color: #FFFFFF;
    opacity: 0.9;
  }
}
</style>
