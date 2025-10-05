# 🔒 Proteger la Rama `main` - Instrucciones

## ⚠️ IMPORTANTE: Hacer ANTES de trabajar en equipo

### 🌐 Configurar desde GitHub Web (Recomendado)

1. **Ir a:** `https://github.com/[OWNER]/[REPO]/settings/branches`

2. **Click:** "Add branch protection rule"

3. **Branch name pattern:** `main`

4. **Configurar estas reglas:**

#### ✅ Protect matching branches

```
☑️ Require a pull request before merging
   ☑️ Require approvals: 1
   ☑️ Dismiss stale pull request approvals when new commits are pushed
   ☑️ Require review from Code Owners
   ☑️ Restrict who can dismiss pull request reviews
      → Solo: Repository administrators

☑️ Require status checks to pass before merging
   ☑️ Require branches to be up to date before merging
   Status checks: (Seleccionar cuando estén disponibles)
   - ☑️ Project Board Automation
   - ☑️ Tests (cuando se agreguen)
   - ☑️ Build (cuando se configure CI)

☑️ Require conversation resolution before merging

☑️ Require signed commits (Recomendado para seguridad)

☑️ Require linear history
   (Evita merge commits, solo squash o rebase)

☑️ Require deployments to succeed before merging
   (Opcional: configurar cuando haya staging environment)

☑️ Lock branch
   ☑️ Make the branch read-only
      → Solo permitir cambios via PR

☑️ Do not allow bypassing the above settings
   (Nadie puede saltarse las reglas, ni admins)

☑️ Restrict who can push to matching branches
   → Seleccionar: Repository administrators, maintainers
   → Excluir: write/triage roles

☑️ Include administrators
   (Para que TÚ también sigas las reglas - CRÍTICO)

☐ Allow force pushes (NUNCA activar)
☐ Allow deletions (NUNCA activar)
```

5. **Click:** "Create" o "Save changes"

---

### 🖥️ Alternativa: Configurar vía GitHub CLI (Avanzado)

Para automatizar la configuración con script:

```bash
# Habilitar protección básica
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_pull_request_reviews[required_approving_review_count]=1 \
  --field required_pull_request_reviews[dismiss_stale_reviews]=true \
  --field required_pull_request_reviews[require_code_owner_reviews]=true \
  --field enforce_admins=true \
  --field required_linear_history=true \
  --field allow_force_pushes=false \
  --field allow_deletions=false \
  --field required_conversation_resolution=true \
  --field lock_branch=true

# Ver configuración actual
gh api repos/:owner/:repo/branches/main/protection | jq
```

**Nota:** Reemplazar `:owner` y `:repo` con tus valores reales.

---

## ✅ Verificar que Funciona

Intenta hacer push directo a main:

```bash
# Esto debería FALLAR ❌
git checkout main
echo "test" >> test.txt
git add test.txt
git commit -m "test"
git push origin main

# Deberías ver error:
# remote: error: GH006: Protected branch update failed
```

Si funciona correctamente, **no podrás hacer push directo a main** ✅

---

## 🔄 Ahora el Flujo OBLIGA a usar PRs

```bash
# 1. Crear rama
git checkout -b feature/test

# 2. Hacer cambios y commit
git add .
git commit -m "feat: test"

# 3. Push de la rama
git push origin feature/test

# 4. Crear PR (CLI o web)
gh pr create

# 5. Esperar aprobación
# 6. Mergear desde GitHub web o CLI
gh pr merge [NUM] --squash
```

---

## 📋 Checklist de Seguridad Completo

### Básico (Mínimo requerido)
- [ ] Rama `main` protegida
- [ ] Require PR antes de merge: ✅
- [ ] Require 1 aprobación mínima: ✅
- [ ] Include administrators: ✅
- [ ] Force push disabled: ✅
- [ ] Deletions disabled: ✅

### Intermedio (Recomendado)
- [ ] Conversation resolution required: ✅
- [ ] Dismiss stale PR approvals: ✅
- [ ] Require review from Code Owners: ✅
- [ ] Require branches up to date: ✅
- [ ] Status checks configurados: ✅
- [ ] Linear history (squash/rebase only): ✅

### Avanzado (Máxima seguridad)
- [ ] Require signed commits: ✅
- [ ] Lock branch (read-only): ✅
- [ ] No bypassing settings: ✅
- [ ] Restrict who can push: ✅
- [ ] Restrict who can dismiss reviews: ✅
- [ ] Require deployments: ✅ (si aplica)

---

## 🚨 Bypass de Emergencia (Solo si es CRÍTICO)

Si necesitas hacer un hotfix urgente y no puedes esperar review:

1. Ir a: `https://github.com/[OWNER]/[REPO]/settings/branches`
2. Desactivar temporalmente "Include administrators"
3. Hacer el push de emergencia
4. **REACTIVAR inmediatamente** "Include administrators"

**⚠️ Solo usar en emergencias reales.**

---

## 🔐 Mejores Prácticas Adicionales

### 1. Configurar Signed Commits (GPG)

Los commits firmados verifican la identidad del autor:

```bash
# Generar clave GPG (si no tienes)
gpg --full-generate-key

# Listar claves
gpg --list-secret-keys --keyid-format=long

# Configurar Git para firmar
git config --global user.signingkey [KEY_ID]
git config --global commit.gpgsign true

# Agregar clave a GitHub
gpg --armor --export [KEY_ID]
# Copiar output y agregarlo en: https://github.com/settings/keys
```

### 2. Proteger Otras Ramas Importantes

Si usas ramas de desarrollo o staging:

```bash
# Proteger rama develop (menos restrictivo)
# Permitir: develop → main (via PR)
# Requerir: 1 aprobación, status checks
```

### 3. Configurar Webhooks de Seguridad

Monitorear eventos sospechosos:

- Push force detectado
- Cambios en configuración de protección
- Eliminación de branches

### 4. Rulesets (Nueva Feature GitHub)

Alternativa moderna a branch protection:

1. Ir a: `https://github.com/[OWNER]/[REPO]/settings/rules`
2. Create ruleset
3. Aplicar a: `main`, `develop`, `release/*`
4. Ventajas: más granular, mejor UI, bypass roles

---

## 📚 Referencias

- [GitHub Branch Protection Docs](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [Signed Commits Guide](https://docs.github.com/en/authentication/managing-commit-signature-verification)
- [Repository Rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets)

---

**Siguiente paso:** Leer `docs/workflow/01-team-workflow.md` para el flujo completo de trabajo en equipo.
