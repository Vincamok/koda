const createNextIntlPlugin = require('next-intl/plugin')

const withNextIntl = createNextIntlPlugin('./src/i18n.ts')

/** @type {import('next').NextConfig} */
const nextConfig = {
  transpilePackages: ['@koda/shared-types', '@koda/i18n'],
}

module.exports = withNextIntl(nextConfig)
