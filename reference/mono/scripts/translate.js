const path = require('path')
const { TranslateTool } = require('@theblockcrypto/launchpad-translation-tool')

require('dotenv').config()

const exitingPath = path.resolve(__dirname, '../static/locales')
const templatePath = path.resolve(__dirname, '../i18n.template.json')
const enLocaleFilePath = path.resolve(__dirname, '../static/locales/en.json')

const { GOOGLE_KEY_FILE_NAME, GOOGLE_PROJECT_ID } = process.env

TranslateTool(
  GOOGLE_PROJECT_ID,
  GOOGLE_KEY_FILE_NAME,
  exitingPath,
  templatePath,
  enLocaleFilePath
)
