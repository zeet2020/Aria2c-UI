import eleLocaleEn from 'element-ui/lib/locale/lang/en'
import appLocaleEnUS from '@shared/locales/en-US'

// English-only build.
/* eslint-disable quote-props */
const resources = {
  'en-US': {
    translation: {
      ...eleLocaleEn,
      ...appLocaleEnUS
    }
  }
}
/* eslint-enable quote-props */

export default resources
