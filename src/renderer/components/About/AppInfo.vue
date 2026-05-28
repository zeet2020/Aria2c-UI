<template>
  <div class="app-info">
    <div class="app-version">
      <span>Aria2 UI</span>
    </div>
    <div class="engine-info" v-if="!!engine">
      <h4>Aria2c version {{engine.version}}</h4>
      <h4 v-if="tauriVersion">Tauri version {{tauriVersion}}</h4>
      <ul v-if="!isMas()">
        <li
          v-for="(feature, index) in engine.enabledFeatures"
          v-bind:key="`feature-${index}`">
          {{ feature }}
        </li>
      </ul>
    </div>
    <div class="fork-notice">
      <p>
        Aria2 UI is a fork of
        <span class="link" @click="openRepo">Motrix</span>,
        ported to run on <strong>Tauri</strong> instead of Electron — all
        original functionality kept intact.
      </p>
      <p class="repo">
        <span class="link" @click="openRepo">github.com/agalwood/Motrix</span>
      </p>
    </div>
  </div>
</template>

<script>
  import is from 'electron-is'
  import { shell } from '@electron/remote'
  import { invoke } from '@tauri-apps/api/core'

  export default {
    name: 'mo-app-info',
    props: {
      version: {
        type: String,
        default: ''
      },
      engine: {
        type: Object,
        default () {
          return {
            version: '',
            enabledFeatures: []
          }
        }
      }
    },
    data () {
      return {
        tauriVersion: ''
      }
    },
    created () {
      invoke('get_tauri_version')
        .then((v) => { this.tauriVersion = v })
        .catch(() => {})
    },
    methods: {
      isMas: is.mas,
      openRepo () {
        shell.openExternal('https://github.com/agalwood/Motrix')
      }
    }
  }
</script>

<style lang="scss">
.app-info {
  position: relative;
  margin: 8px 0;
  .app-version {
    text-align: center;
    span {
      font-size: $--font-size-large;
      font-weight: 700;
      color: $--app-version-color;
      line-height: 18px;
    }
    .version-num {
      font-weight: normal;
      margin-left: 16px;
      opacity: 0.8;
    }
  }
  .fork-notice {
    margin: 16px 8px 0;
    font-size: 12px;
    line-height: 18px;
    color: $--app-engine-info-color;
    p {
      margin: 4px 0;
    }
    .repo {
      opacity: 0.85;
    }
    .link {
      color: $--color-primary;
      cursor: pointer;
      &:hover {
        text-decoration: underline;
      }
    }
  }
  .engine-info {
    margin: 50px 35% 0 8px;
    h4 {
      font-size: $--font-size-base;
      font-weight: normal;
      color: $--app-engine-title-color;
    }
    ul {
      font-size: 12px;
      color: $--app-engine-info-color;
      list-style: none;
      padding: 0;
      line-height: 20px;
      @include clearfix();
      li {
        float: left;
        width: 50%;
      }
    }
  }
}
</style>
