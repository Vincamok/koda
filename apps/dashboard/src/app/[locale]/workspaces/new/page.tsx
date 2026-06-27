'use client'

import * as React from 'react'
import { useRouter } from 'next/navigation'
import { useTranslations } from 'next-intl'
import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { createWorkspace } from '@/lib/api-client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { useToast } from '@/components/ui/toast'

interface NewWorkspacePageProps {
  params: { locale: string }
}

const TEMPLATES = [
  { value: 'blank', label: 'Blank' },
  { value: 'node', label: 'Node.js' },
  { value: 'python', label: 'Python' },
  { value: 'rust', label: 'Rust' },
  { value: 'go', label: 'Go' },
  { value: 'java', label: 'Java' },
]

export default function NewWorkspacePage({ params: { locale } }: NewWorkspacePageProps) {
  const t = useTranslations('workspace.create')
  const router = useRouter()
  const { addToast } = useToast()

  const [name, setName] = React.useState('')
  const [gitUrl, setGitUrl] = React.useState('')
  const [branch, setBranch] = React.useState('main')
  const [templateId, setTemplateId] = React.useState('blank')
  const [loading, setLoading] = React.useState(false)
  const [errors, setErrors] = React.useState<{
    name?: string
    gitUrl?: string
    branch?: string
  }>({})

  const validate = (): boolean => {
    const next: typeof errors = {}
    if (!name.trim()) next.name = 'Nom requis'
    else if (name.length > 64) next.name = '64 caractères maximum'
    if (gitUrl && !/^https?:\/\/.+/.test(gitUrl) && !/^git@.+/.test(gitUrl)) {
      next.gitUrl = 'URL Git invalide'
    }
    if (!branch.trim()) next.branch = 'Branche requise'
    setErrors(next)
    return Object.keys(next).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validate()) return

    setLoading(true)
    try {
      const workspace = await createWorkspace('default', {
        name: name.trim(),
        git_url: gitUrl || undefined,
        branch: branch || undefined,
        template_id: templateId || undefined,
      })
      addToast(`Workspace "${workspace.name}" créé`, 'success')
      router.push(`/${locale}/workspaces`)
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Erreur lors de la création'
      addToast(message, 'error')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-koda-surface">
      {/* Top bar */}
      <div className="flex h-14 items-center border-b border-koda-border px-6">
        <Link
          href={`/${locale}/workspaces`}
          className="flex items-center gap-2 text-sm text-koda-text-muted hover:text-koda-text transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Retour
        </Link>
      </div>

      {/* Form */}
      <div className="flex flex-1 items-start justify-center p-6">
        <div className="w-full max-w-lg">
          <div className="mb-6">
            <h1 className="text-2xl font-bold text-koda-text">{t('title')}</h1>
            <p className="mt-1 text-sm text-koda-text-muted">
              Configurez votre nouvel environnement de développement.
            </p>
          </div>

          <form
            onSubmit={handleSubmit}
            className="rounded-xl border border-koda-border bg-koda-surface-raised p-6 space-y-5"
          >
            {/* Name */}
            <div className="space-y-1.5">
              <Label htmlFor="ws-name">{t('name')}</Label>
              <Input
                id="ws-name"
                type="text"
                placeholder={t('name_placeholder')}
                value={name}
                onChange={(e) => setName(e.target.value)}
                error={errors.name}
                disabled={loading}
                maxLength={64}
              />
            </div>

            {/* Git URL */}
            <div className="space-y-1.5">
              <Label htmlFor="ws-git">
                {t('git_url')}{' '}
                <span className="text-koda-text-muted font-normal">(optionnel)</span>
              </Label>
              <Input
                id="ws-git"
                type="url"
                placeholder={t('git_url_placeholder')}
                value={gitUrl}
                onChange={(e) => setGitUrl(e.target.value)}
                error={errors.gitUrl}
                disabled={loading}
              />
            </div>

            {/* Branch */}
            <div className="space-y-1.5">
              <Label htmlFor="ws-branch">{t('branch')}</Label>
              <Input
                id="ws-branch"
                type="text"
                placeholder={t('branch_placeholder')}
                value={branch}
                onChange={(e) => setBranch(e.target.value)}
                error={errors.branch}
                disabled={loading}
              />
            </div>

            {/* Template */}
            <div className="space-y-1.5">
              <Label htmlFor="ws-template">{t('template')}</Label>
              <Select
                id="ws-template"
                options={TEMPLATES}
                value={templateId}
                onValueChange={setTemplateId}
                placeholder={t('template_placeholder')}
                disabled={loading}
              />
            </div>

            {/* Actions */}
            <div className="flex items-center gap-3 pt-2">
              <Button
                type="submit"
                variant="primary"
                size="md"
                disabled={loading}
                className="flex-1"
              >
                {loading ? t('submitting') : t('submit')}
              </Button>
              <Link href={`/${locale}/workspaces`} className="flex-1">
                <Button
                  type="button"
                  variant="secondary"
                  size="md"
                  disabled={loading}
                  className="w-full"
                >
                  {t('cancel')}
                </Button>
              </Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}
