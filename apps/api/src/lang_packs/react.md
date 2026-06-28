# React Framework Pack

You are working with **React 18** with hooks and functional components.

## Hooks best practices

```tsx
// State
const [value, setValue] = React.useState<string>('')

// Effects — always specify deps, clean up subscriptions
React.useEffect(() => {
  const sub = subscribe()
  return () => sub.unsubscribe()
}, [dependency])

// Refs for imperative DOM access
const ref = React.useRef<HTMLDivElement>(null)

// Memoization — only when profiling shows a problem
const computed = React.useMemo(() => expensive(data), [data])
const cb = React.useCallback(() => doThing(id), [id])
```

## Component patterns
- Prefer named exports over default exports for tree-shaking
- Co-locate types with components
- Lift state only as high as needed
- Use composition over inheritance

## Performance
- `React.memo()` for pure components that receive stable props
- Avoid creating objects/arrays inline in JSX (new reference on every render)
- Use `key` prop correctly — stable, unique, never array index for reordered lists

## Event handling
```tsx
const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
  e.preventDefault()
  // ...
}
const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
  setValue(e.target.value)
}
```

## Async patterns
```tsx
// Prefer AbortController for cancellable fetches
const controller = new AbortController()
const res = await fetch(url, { signal: controller.signal })
// cleanup: controller.abort()
```

## TypeScript
- Type props explicitly — avoid `any`
- Use `React.ReactNode` for children prop
- Discriminated unions for component variants
