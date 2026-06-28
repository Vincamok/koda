# Next.js Framework Pack

You are working with **Next.js 14** (App Router, React Server Components).

## App Router fundamentals

```
app/
  layout.tsx        — RootLayout, wraps all pages
  page.tsx          — Server Component by default
  loading.tsx       — Suspense boundary
  error.tsx         — Error boundary ('use client')
  [param]/page.tsx  — Dynamic segment
```

## Server vs Client Components
- Server Components: no hooks, no browser APIs, can `await` directly, access DB
- Client Components: add `'use client'` directive, can use hooks and browser APIs
- Prefer server components; push interactivity to leaves

```tsx
// Server Component — no 'use client'
export default async function Page() {
  const data = await db.query()  // ← direct DB access OK
  return <ClientWidget initialData={data} />
}

// Client Component
'use client'
export function ClientWidget({ initialData }) {
  const [state, setState] = useState(initialData)
  // ...
}
```

## Data fetching (App Router)
```tsx
// Cache by default, revalidate on demand
const data = await fetch(url, { next: { revalidate: 60 } })
// No cache (dynamic)
const data = await fetch(url, { cache: 'no-store' })
```

## next-intl (i18n)
```tsx
// Server
import { getTranslations } from 'next-intl/server'
const t = await getTranslations('namespace')
// Client
import { useTranslations } from 'next-intl'
const t = useTranslations('namespace')
```

## Route Handlers
```tsx
// app/api/route.ts
export async function GET(request: Request) {
  return Response.json({ data })
}
```

## Best practices
- Use `redirect()` from `next/navigation` in server components
- Use `useRouter()` from `next/navigation` in client components
- Prefer `Link` over `<a>` for internal navigation
- Images: use `next/image` for optimization
- Metadata: export `metadata` object or `generateMetadata` function
