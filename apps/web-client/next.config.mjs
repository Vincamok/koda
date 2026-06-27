import createNextIntlPlugin from 'next-intl/plugin'

const withNextIntl = createNextIntlPlugin('./src/i18n.ts')

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  transpilePackages: ['@koda/api-client', '@koda/i18n', '@koda/shared-types'],
  webpack: (config) => {
    // Monaco editor needs these
    config.resolve.alias = {
      ...config.resolve.alias,
    }
    return config
  },
}

export default withNextIntl(nextConfig)
